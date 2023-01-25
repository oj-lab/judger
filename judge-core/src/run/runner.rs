use libc::{rusage, wait4};
use log::{error, info};
use nix::{
    fcntl::{self, fcntl},
    sys::{
        epoll::{epoll_create, epoll_ctl, epoll_wait, EpollEvent, EpollFlags, EpollOp},
        wait::waitpid,
    },
    unistd::{close, fork, pipe, ForkResult},
};

use crate::{
    error::JudgeCoreError,
    utils::{
        get_default_rusage,
        io::{read_string_from_fd, write_str_to_fd},
    },
};

use super::judger::run_judger;

pub fn run() -> Result<(), JudgeCoreError> {
    // build pipes here
    let (judge_in_rfd, judge_in_wfd) = pipe()?;
    let (judge_out_rfd, judge_out_wfd) = pipe()?;
    let (test_in_rfd, test_in_wfd) = pipe()?;
    let (test_out_rfd, test_out_wfd) = pipe()?;
    let (signal_rfd, signal_wfd) = pipe()?;

    info!(
        "created pipes with... \
        judge_in_rfd: {}, judge_in_wfd: {}, \
        judge_out_rfd: {}, judge_out_wfd: {}, \
        test_in_rfd: {}, test_in_wfd: {}, \
        test_out_rfd: {}, test_out_wfd: {}",
        judge_in_rfd,
        judge_in_wfd,
        judge_out_rfd,
        judge_out_wfd,
        test_in_rfd,
        test_in_wfd,
        test_out_rfd,
        test_out_wfd
    );
    // set nonblock for monitor side fd
    fcntl(judge_out_rfd, nix::fcntl::F_SETFL(fcntl::OFlag::O_NONBLOCK))?;
    fcntl(test_out_rfd, nix::fcntl::F_SETFL(fcntl::OFlag::O_NONBLOCK))?;

    let epoll_fd = epoll_create()?;
    const SIGNAL_DATA: u64 = 0;
    const JUDGE_OUT_DATA: u64 = 1;
    const TEST_OUT_DATA: u64 = 2;
    const JUDGE_EXIT_SIGNAL: u8 = 1;
    const TEST_EXIT_SIGNAL: u8 = 2;
    epoll_ctl(
        epoll_fd,
        EpollOp::EpollCtlAdd,
        signal_rfd,
        &mut EpollEvent::new(EpollFlags::EPOLLIN | EpollFlags::EPOLLET, SIGNAL_DATA),
    )?;
    epoll_ctl(
        epoll_fd,
        EpollOp::EpollCtlAdd,
        judge_out_rfd,
        &mut EpollEvent::new(EpollFlags::EPOLLIN | EpollFlags::EPOLLET, JUDGE_OUT_DATA),
    )?;
    epoll_ctl(
        epoll_fd,
        EpollOp::EpollCtlAdd,
        test_out_rfd,
        &mut EpollEvent::new(EpollFlags::EPOLLIN | EpollFlags::EPOLLET, TEST_OUT_DATA),
    )?;

    let test_string = "test line_1\ntest_line_2\n\ntest_line_4\n";
    write_str_to_fd(judge_in_wfd, test_string, true)?;

    let mut status = 0;
    let mut usage: rusage = get_default_rusage();

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            let judge_pid = child;
            match unsafe { fork() } {
                Ok(ForkResult::Parent { child }) => {
                    let test_pid = child;
                    info!(
                        "forked child processes with judge_pid: {} and test_pid: {}",
                        judge_pid, test_pid
                    );

                    let mut wait_test_result;
                    let mut wait_judge_status;
                    let mut events = [
                        EpollEvent::new(EpollFlags::empty(), 0),
                        EpollEvent::new(EpollFlags::empty(), 0),
                        EpollEvent::new(EpollFlags::empty(), 0),
                    ];
                    let mut judge_exit = false;
                    let mut test_exit = false;
                    loop {
                        let events_num = epoll_wait(epoll_fd, &mut events, -1)?;
                        for i in 0..events_num {
                            let event = events[i];
                            info!(
                                "got epoll event data: {}, flags: {:?}",
                                event.data(),
                                event.events()
                            );
                            match event.data() {
                                SIGNAL_DATA => {
                                    let mut buffer: [u8; 1] = [0; 1];
                                    match nix::unistd::read(signal_rfd, &mut buffer) {
                                        Ok(_len) => match buffer[0] {
                                            JUDGE_EXIT_SIGNAL => {
                                                info!("got judge exit signal");
                                                wait_judge_status = waitpid(judge_pid, None)?;
                                                info!("wait judge status: {:?}", wait_judge_status);
                                                judge_exit = true;
                                            }
                                            TEST_EXIT_SIGNAL => {
                                                info!("got test exit signal");
                                                unsafe {
                                                    wait_test_result = wait4(
                                                        test_pid.as_raw() as i32,
                                                        &mut status,
                                                        0,
                                                        &mut usage,
                                                    );
                                                }
                                                info!(
                                                    "wait test result: {}, usage: {:?}",
                                                    wait_test_result, usage
                                                );
                                                test_exit = true;
                                            }
                                            _ => {
                                                info!("unknown signal");
                                            }
                                        },
                                        Err(e) => {
                                            error!("read signal failed with error: {:?}", e);
                                            return Err(JudgeCoreError::NixErrno(e));
                                        }
                                    }
                                }
                                JUDGE_OUT_DATA => {
                                    transfer_data(judge_out_rfd, test_in_wfd)?;
                                }
                                TEST_OUT_DATA => {
                                    transfer_data(test_out_rfd, judge_in_wfd)?;
                                }
                                _ => {
                                    info!("unknown data");
                                }
                            }
                        }
                        if judge_exit && test_exit {
                            break;
                        }
                    }

                    // close pipes here
                    close(judge_in_rfd)?;
                    close(judge_in_wfd)?;
                    close(judge_out_rfd)?;
                    close(judge_out_wfd)?;
                    close(test_in_rfd)?;
                    close(test_in_wfd)?;
                    close(test_out_rfd)?;
                    close(test_out_wfd)?;
                    close(signal_rfd)?;
                    close(signal_wfd)?;
                    // close epoll fd
                    close(epoll_fd)?;

                    return Ok(());
                }
                Ok(ForkResult::Child) => {
                    // run test program here
                    match nix::unistd::write(signal_wfd, &[TEST_EXIT_SIGNAL]) {
                        Ok(_len) => {
                            info!("write test exit signal");
                        }
                        Err(e) => {
                            error!("write test signal failed with error: {:?}", e);
                            return Err(JudgeCoreError::NixErrno(e));
                        }
                    }

                    return Ok(());
                }
                Err(e) => {
                    error!("fork failed with error: {:?}", e);
                    return Err(JudgeCoreError::NixErrno(e));
                }
            }
        }
        Ok(ForkResult::Child) => {
            // run judge program here
            info!("run_judger");
            run_judger(super::RunConfig {
                program_path: None,
                input_fd: judge_in_rfd,
                output_fd: judge_out_wfd,
                rlimit_config: None,
            })?;

            match nix::unistd::write(signal_wfd, &[JUDGE_EXIT_SIGNAL]) {
                Ok(_len) => {
                    info!("write judge exit signal");
                }
                Err(e) => {
                    error!("write judge signal failed with error: {:?}", e);
                    return Err(JudgeCoreError::NixErrno(e));
                }
            }

            return Ok(());
        }
        Err(e) => {
            error!("fork failed with error: {:?}", e);
            return Err(JudgeCoreError::NixErrno(e));
        }
    }
}

fn transfer_data(from_fd: i32, to_fd: i32) -> Result<String, JudgeCoreError> {
    let data = read_string_from_fd(from_fd, false)?;
    if data != "" {
        write_str_to_fd(to_fd, &data, false)?;
        info!(
            "transfered {} bytes of test out from fd:{} to fd:{}",
            data.as_bytes().len(),
            from_fd,
            to_fd
        );
    }
    Ok(data)
}

#[cfg(test)]
pub mod runner {
    use log::LevelFilter;

    use super::*;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Info)
            .try_init();
    }

    #[test]
    fn test_run() {
        init();
        run().expect("error");
    }
}
