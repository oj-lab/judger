use crate::error::JudgeCoreError;
use crate::sandbox::{ProcessListener, RawRunResultInfo, ResourceLimitConfig, SandBox};
use nix::errno::Errno;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::sys::epoll::{
    epoll_create1, epoll_ctl, epoll_wait, EpollCreateFlags, EpollEvent, EpollFlags, EpollOp,
};
use nix::unistd::{pipe, read, write};
use std::fs::File;
use std::os::unix::io::{AsRawFd, RawFd};

pub struct RunnerConfig {
    pub program_path: String,
    pub checker_path: String,
    pub input_file_path: String,
    pub output_file_path: String,
    pub answer_file_path: String,
    pub rlimit_config: ResourceLimitConfig,
}

pub fn run_judge(runner_config: &RunnerConfig) -> Result<Option<RawRunResultInfo>, JudgeCoreError> {
    let mut user_process = SandBox::new(true)?;
    let input_file = File::open(&runner_config.input_file_path)?;
    let output_file = File::options()
        .write(true)
        .truncate(true) // Overwrite the whole content of this file
        .open(&runner_config.output_file_path)
        .unwrap();
    let input_raw_fd: RawFd = input_file.as_raw_fd();
    let output_raw_fd: RawFd = output_file.as_raw_fd();
    let user_spawn = user_process.spawn_with_io(
        &runner_config.program_path,
        &[&String::from("")],
        &runner_config.rlimit_config,
        input_raw_fd,
        output_raw_fd,
    )?;
    if user_spawn.is_none() {
        return Ok(None);
    }
    let _user_result = user_process.wait()?;

    let mut checker_process = SandBox::new(false)?;
    let first_args = String::from("");
    let checker_args = vec![
        &first_args,
        &runner_config.input_file_path,
        &runner_config.output_file_path,
        &runner_config.answer_file_path,
    ];

    let checker_spawn = checker_process.spawn(
        &runner_config.checker_path,
        &checker_args,
        &runner_config.rlimit_config,
    )?;
    if checker_spawn.is_none() {
        return Ok(None);
    }
    let checker_result = checker_process.wait()?;
    Ok(checker_result)
}

fn set_non_blocking(fd: RawFd) {
    fcntl(fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).expect("failed to set fd to non blocking");
}

// write the content of `from` to `to`, record to output
fn pump_proxy_pipe(from: RawFd, to: RawFd, output: RawFd) {
    let mut buf = [0; 1024];
    loop {
        match read(from, &mut buf) {
            Ok(nread) => {
                println!("{} read. {} -> {}", nread, from, to);
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

pub fn run_interact(
    runner_config: &RunnerConfig,
    interactor_path: &str,
    output_path: &String,
) -> Result<Option<RawRunResultInfo>, JudgeCoreError> {
    let mut user_process = ProcessListener::new(true)?;
    let mut interact_process = ProcessListener::new(false)?;

    fn create_pipe() -> (RawFd, RawFd) {
        pipe().expect("Failed to create pipe")
    }

    let (proxy_read_user, user_write_proxy) = create_pipe();
    let (proxy_read_interactor, interactor_write_proxy) = create_pipe();
    let (user_read_proxy, proxy_write_user) = create_pipe();
    let (interactor_read_proxy, proxy_write_interactor) = create_pipe();

    // epoll will listen to the write event
    // when should it be non blocking???
    set_non_blocking(user_write_proxy);
    set_non_blocking(interactor_write_proxy);
    set_non_blocking(proxy_read_user);
    set_non_blocking(proxy_read_interactor);

    let epoll_fd =
        epoll_create1(EpollCreateFlags::EPOLL_CLOEXEC).expect("Failed to create epoll instance");

    fn add_fd(epoll_fd: RawFd, fd: RawFd) {
        let mut event = EpollEvent::new(EpollFlags::EPOLLIN, fd as u64);
        epoll_ctl(epoll_fd, EpollOp::EpollCtlAdd, fd, Some(&mut event))
            .expect("Failed to add fd to epoll");
    }

    add_fd(epoll_fd, proxy_read_user);
    add_fd(epoll_fd, proxy_read_interactor);

    let (user_exit_read, user_exit_write) = create_pipe();
    let (interactor_exit_read, interactor_exit_write) = create_pipe();

    add_fd(epoll_fd, user_exit_read);
    add_fd(epoll_fd, interactor_exit_read);
    user_process.set_exit_fd(user_exit_write, 41u8);
    interact_process.set_exit_fd(interactor_exit_write, 42u8);

    let output_file = File::options()
        .write(true)
        .truncate(true) // Overwrite the whole content of this file
        .open(output_path)
        .unwrap();
    let output_raw_fd: RawFd = output_file.as_raw_fd();
    println!("Spawning user process");
    let user_spawn = user_process.spawn_with_io(
        &runner_config.program_path,
        &[&String::from("")],
        &runner_config.rlimit_config,
        user_read_proxy,
        user_write_proxy,
    )?;

    if user_spawn.is_none() {
        return Ok(None);
    }

    let first_args = String::from("");
    let interact_args = vec![
        &first_args,
        &runner_config.input_file_path,
        &runner_config.output_file_path,
        &runner_config.answer_file_path,
    ];
    println!("Spawning interactor process");
    let interact_spawn = interact_process.spawn_with_io(
        interactor_path,
        &interact_args,
        &runner_config.rlimit_config,
        interactor_read_proxy,
        interactor_write_proxy,
    )?;

    if interact_spawn.is_none() {
        return Ok(None);
    }

    let mut events = [EpollEvent::empty(); 128];
    loop {
        let num_events = epoll_wait(epoll_fd, &mut events, -1).expect("Failed to wait for events");
        println!("{} events found!", num_events);
        let mut exited = false;
        for event in events.iter().take(num_events) {
            let fd = event.data() as RawFd;
            if fd == user_exit_read || fd == interactor_exit_read {
                println!("{:?} fd exited", fd);
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

    println!("Epoll finished!");

    // TODO: get result from listener
    // let _user_result = user_process.wait()?;
    // let _interact_result = interact_process.wait()?;

    let mut checker_process = SandBox::new(false)?;
    // the checker will compare the output of interactor with answer file
    let checker_args = vec![
        &first_args,
        &runner_config.input_file_path,
        &runner_config.output_file_path,
        &runner_config.answer_file_path,
    ];
    println!("Spawning checker process");
    let checker_spawn = checker_process.spawn(
        &runner_config.checker_path,
        &checker_args,
        &runner_config.rlimit_config,
    )?;

    if checker_spawn.is_none() {
        return Ok(None);
    }
    let checker_result = checker_process.wait()?;
    Ok(checker_result)
}

#[cfg(test)]
pub mod monitor {
    use super::*;
    use crate::sandbox::ResourceLimitConfig;

    const TEST_CONFIG: ResourceLimitConfig = ResourceLimitConfig {
        stack_limit: Some((64 * 1024 * 1024, 64 * 1024 * 1024)),
        as_limit: Some((256 * 1024 * 1024, 256 * 1024 * 1024)),
        cpu_limit: Some((1, 2)),
        nproc_limit: Some((1, 1)),
        fsize_limit: Some((1024, 1024)),
    };

    #[test]
    fn test_run_judge() {
        let runner_config = RunnerConfig {
            program_path: "./../test-collection/programs/read_and_write".to_owned(),
            checker_path: "./../test-collection/programs/checkers/lcmp".to_owned(),
            input_file_path: "../tmp/in".to_owned(),
            output_file_path: "../tmp/out".to_owned(),
            answer_file_path: "../tmp/ans".to_owned(),
            rlimit_config: TEST_CONFIG,
        };
        let result = run_judge(&runner_config).expect("error").unwrap();
        println!("{:?}", result);
    }

    #[test]
    fn test_run_interact() {
        let runner_config = RunnerConfig {
            program_path: "./../test-collection/programs/read_and_write".to_owned(),
            checker_path: "./../test-collection/programs/checkers/lcmp".to_owned(),
            input_file_path: "../tmp/in".to_owned(),
            output_file_path: "../tmp/out".to_owned(),
            answer_file_path: "../tmp/ans".to_owned(),
            rlimit_config: TEST_CONFIG,
        };
        let result = run_interact(
            &runner_config,
            &String::from("../test-collection/programs/checkers/interactor-a-plus-b"),
            &String::from("../tmp/interactor"),
        )
        .expect("error");
        if !result.is_none() {
            println!("{:?}", result);
        }
    }
}
