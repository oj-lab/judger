use crate::compiler::Language;
use crate::error::JudgeCoreError;
use crate::executor::Executor;
use crate::sandbox::{ProcessListener, RawRunResultInfo, SandBox, SCRIPT_LIMIT_CONFIG};

use nix::errno::Errno;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::sys::epoll::{
    epoll_create1, epoll_ctl, epoll_wait, EpollCreateFlags, EpollEvent, EpollFlags, EpollOp,
};
use nix::unistd::{pipe, read, write};
use std::fs::File;
use std::os::unix::io::{AsRawFd, RawFd};

use super::JudgeConfig;

fn set_non_blocking(fd: RawFd) -> Result<libc::c_int, JudgeCoreError> {
    log::debug!("Setting fd={} to non blocking", fd);
    Ok(fcntl(fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK))?)
}

// write the content of `from` to `to`, record to output
fn pump_proxy_pipe(from: RawFd, to: RawFd, output: RawFd) {
    let mut buf = [0; 1024];
    loop {
        match read(from, &mut buf) {
            Ok(nread) => {
                log::debug!("{} read. {} -> {}", nread, from, to);
                write(to, &buf[..nread]).ok();
                write(output, &buf[..nread]).ok();
            }
            Err(e) => {
                if e == Errno::EAGAIN || e == Errno::EWOULDBLOCK {
                    return;
                }
                panic!("failed to read from pipe");
            }
        }
    }
}

fn add_epoll_fd(epoll_fd: RawFd, fd: RawFd) -> Result<(), JudgeCoreError> {
    let mut event = EpollEvent::new(EpollFlags::EPOLLIN, fd as u64);
    log::debug!("Adding fd={} to epoll", fd);
    Ok(epoll_ctl(
        epoll_fd,
        EpollOp::EpollCtlAdd,
        fd,
        Some(&mut event),
    )?)
}

pub fn run_interact(
    runner_config: &JudgeConfig,
    interactor_path: &str,
    output_path: &String,
) -> Result<Option<RawRunResultInfo>, JudgeCoreError> {
    log::debug!("Creating sandbox for user process");
    let mut user_process = ProcessListener::new(true)?;
    log::debug!("Creating sandbox for interactor process");
    let mut interact_process = ProcessListener::new(false)?;

    fn create_pipe() -> Result<(RawFd, RawFd), JudgeCoreError> {
        log::debug!("Creating pipe");
        Ok(pipe()?)
    }

    log::debug!("Creating pipes");
    let (proxy_read_user, user_write_proxy) = create_pipe()?;
    let (proxy_read_interactor, interactor_write_proxy) = create_pipe()?;
    let (user_read_proxy, proxy_write_user) = create_pipe()?;
    let (interactor_read_proxy, proxy_write_interactor) = create_pipe()?;

    // epoll will listen to the write event
    // when should it be non blocking???
    log::debug!("Setting pipes to non blocking");
    set_non_blocking(user_write_proxy)?;
    set_non_blocking(interactor_write_proxy)?;
    set_non_blocking(proxy_read_user)?;
    set_non_blocking(proxy_read_interactor)?;

    log::debug!("Creating epoll");
    let epoll_fd = epoll_create1(EpollCreateFlags::EPOLL_CLOEXEC)?;

    log::debug!("Adding fds to epoll");
    add_epoll_fd(epoll_fd, proxy_read_user)?;
    add_epoll_fd(epoll_fd, proxy_read_interactor)?;

    log::debug!("Creating exit pipes");
    let (user_exit_read, user_exit_write) = create_pipe()?;
    let (interactor_exit_read, interactor_exit_write) = create_pipe()?;

    log::debug!("Adding exit fds to epoll");
    add_epoll_fd(epoll_fd, user_exit_read)?;
    add_epoll_fd(epoll_fd, interactor_exit_read)?;
    user_process.set_exit_fd(user_exit_write, 41u8);
    interact_process.set_exit_fd(interactor_exit_write, 42u8);

    log::debug!("Opening input file path={}", runner_config.input_file_path);
    let output_file = File::options()
        .write(true)
        .truncate(true) // Overwrite the whole content of this file
        .open(output_path)?;
    let output_raw_fd: RawFd = output_file.as_raw_fd();
    log::debug!("Spawning user process");
    let user_executor = Executor::new(
        runner_config.language,
        runner_config.program_path.to_owned(),
        vec![String::from("")],
    );
    let user_spawn = user_process.spawn_with_io(
        user_executor,
        &runner_config.rlimit_config,
        user_read_proxy,
        user_write_proxy,
    )?;

    if user_spawn.is_none() {
        return Ok(None);
    }

    let first_args = String::from("");
    let interact_args = vec![
        first_args,
        runner_config.input_file_path.to_owned(),
        runner_config.output_file_path.to_owned(),
        runner_config.answer_file_path.to_owned(),
    ];
    let interact_executor =
        Executor::new(Language::Cpp, interactor_path.to_string(), interact_args);
    log::debug!("Spawning interactor process");
    let interact_spawn = interact_process.spawn_with_io(
        interact_executor,
        &SCRIPT_LIMIT_CONFIG,
        interactor_read_proxy,
        interactor_write_proxy,
    )?;

    if interact_spawn.is_none() {
        return Ok(None);
    }

    let mut events = [EpollEvent::empty(); 128];
    loop {
        let num_events = epoll_wait(epoll_fd, &mut events, -1)?;
        log::debug!("{} events found!", num_events);
        let mut exited = false;
        for event in events.iter().take(num_events) {
            let fd = event.data() as RawFd;
            if fd == user_exit_read || fd == interactor_exit_read {
                log::debug!("{:?} fd exited", fd);
                exited = true;
                break;
            }
            if fd == proxy_read_user {
                pump_proxy_pipe(proxy_read_user, proxy_write_interactor, output_raw_fd);
            } else if fd == proxy_read_interactor {
                pump_proxy_pipe(proxy_read_interactor, proxy_write_user, output_raw_fd);
            }
        }
        if exited {
            break;
        }
    }

    log::debug!("Epoll finished!");

    // TODO: get result from listener
    // let _user_result = user_process.wait()?;
    // let _interact_result = interact_process.wait()?;
    log::debug!("Creating sandbox for checker process");
    if let Some(checker_path) = runner_config.custom_checker_path.clone() {
        let mut checker_process = SandBox::new(false)?;
        let first_args = String::from("");
        let checker_args = vec![
            first_args,
            runner_config.input_file_path.to_owned(),
            runner_config.output_file_path.to_owned(),
            runner_config.answer_file_path.to_owned(),
            runner_config.check_file_path.to_owned(),
        ];
        let checker_executor = Executor::new(Language::Cpp, checker_path, checker_args);
        log::debug!("Spawning checker process");
        let checker_spawn = checker_process.spawn(checker_executor, &SCRIPT_LIMIT_CONFIG)?;
        if checker_spawn.is_none() {
            return Ok(None);
        }
        log::debug!("Waiting for checker process");
        let checker_result = checker_process.wait()?;
        return Ok(Some(checker_result));
    }
    Err(JudgeCoreError::AnyhowError(anyhow::anyhow!(
        "Checker path is not provided"
    )))
}

#[cfg(test)]
pub mod interact_judge_test {
    use crate::{compiler::Language, judge::JudgeConfig, sandbox::ResourceLimitConfig};

    use super::run_interact;

    const TEST_CONFIG: ResourceLimitConfig = ResourceLimitConfig {
        stack_limit: Some((64 * 1024 * 1024, 64 * 1024 * 1024)),
        as_limit: Some((64 * 1024 * 1024, 64 * 1024 * 1024)),
        cpu_limit: Some((1, 2)),
        nproc_limit: Some((1, 1)),
        fsize_limit: Some((1024, 1024)),
    };

    #[test]
    fn test_run_interact() {
        let runner_config = JudgeConfig {
            language: Language::Cpp,
            program_path: "./../test-collection/dist/programs/read_and_write".to_owned(),
            custom_checker_path: Some("./../test-collection/dist/checkers/lcmp".to_owned()),
            input_file_path: "../tmp/in".to_owned(),
            output_file_path: "../tmp/out".to_owned(),
            answer_file_path: "../tmp/ans".to_owned(),
            check_file_path: "../tmp/check".to_owned(),
            rlimit_config: TEST_CONFIG,
        };
        let result = run_interact(
            &runner_config,
            &String::from("../test-collection/dist/checkers/interactor-a-plus-b"),
            &String::from("../tmp/interactor"),
        );
        match result {
            Ok(Some(result)) => {
                log::debug!("{:?}", result);
            }
            Ok(None) => {
                log::debug!("Ignoring this result, for it's from a fork child process");
            }
            Err(e) => {
                log::error!("meet error: {:?}", e);
                assert!(false);
            }
        }
    }
}
