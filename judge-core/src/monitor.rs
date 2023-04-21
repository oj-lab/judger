use crate::error::JudgeCoreError;
use crate::sandbox::{RawRunResultInfo, ResourceLimitConfig, SandBox};
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::sys::epoll::{
    epoll_create1, epoll_ctl, epoll_wait, EpollCreateFlags, EpollEvent, EpollFlags, EpollOp,
};
use nix::unistd::pipe;
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
    let user_process = SandBox::new(true)?;
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
        &vec![&String::from("")],
        &runner_config.rlimit_config,
        input_raw_fd,
        output_raw_fd,
    )?;
    if user_spawn.is_none() {
        return Ok(None);
    }
    let (user_begin, user_pid) = user_spawn.unwrap();
    let user_result = user_process.wait(user_begin, user_pid)?;

    let checker_process = SandBox::new(false)?;
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
    let (check_begin, checker_pid) = checker_spawn.unwrap();
    let checker_result = checker_process.wait(check_begin, checker_pid)?;
    Ok(checker_result)
}

fn set_non_blocking(fd: RawFd) {
    fcntl(fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).expect("failed to set fd to non blocking");
}

pub fn run_interact(
    runner_config: &RunnerConfig,
    interactor_path: &String,
) -> Result<Option<RawRunResultInfo>, JudgeCoreError> {
    let user_process = SandBox::new(true)?;
    let interact_process = SandBox::new(false)?;

    fn create_pipe() -> (RawFd, RawFd) {
        pipe().expect("Failed to create pipe")
    }

    let (proxy_read_fd, proxy_write_fd) = create_pipe();

    // TODO: impl epoll
    // user -> epoll -> interact
    // let epoll_fd = epoll_create1(EpollCreateFlags::EPOLL_CLOEXEC)
    // .expect("Failed to create epoll instance");

    // fn add_fd(epoll_fd: RawFd, fd: RawFd) {
    //     let mut event = EpollEvent::new(EpollFlags::EPOLLIN, fd as u64);
    //     epoll_ctl(epoll_fd, EpollOp::EpollCtlAdd, fd, Some(&mut event))
    //     .expect("Failed to add fd to epoll");
    // }

    let (read_end, write_end) = create_pipe();
    let user_spawn = user_process.spawn_with_io(
        &runner_config.program_path,
        &vec![&String::from("")],
        &runner_config.rlimit_config,
        read_end,
        proxy_write_fd,
    )?;

    let first_args = String::from("");
    let interact_args = vec![
        &first_args,
        &runner_config.input_file_path,
        &runner_config.output_file_path,
        &runner_config.answer_file_path,
    ];

    let interact_spawn = interact_process.spawn_with_io(
        interactor_path,
        &interact_args,
        &runner_config.rlimit_config,
        proxy_read_fd,
        write_end,
    )?;

    if user_spawn.is_none() || interact_spawn.is_none() {
        // ??? Should we return an error here?
        return Ok(None);
    }
    let (user_begin, user_pid) = user_spawn.unwrap();
    let (interact_begin, interact_pid) = interact_spawn.unwrap();
    let _user_result = user_process.wait(user_begin, user_pid)?;
    let _interact_result = interact_process.wait(interact_begin, interact_pid)?;

    let checker_process = SandBox::new(false)?;
    // the checker will compare the output of interactor with answer file
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
    let (checker_begin, checker_pid) = checker_spawn.unwrap();
    let checker_result = checker_process.wait(checker_begin, checker_pid)?;
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
            program_path: "./../test-program/read_and_write".to_owned(),
            checker_path: "./../test-program/checkers/lcmp".to_owned(),
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
            program_path: "./../test-program/read_and_write".to_owned(),
            checker_path: "./../test-program/checkers/lcmp".to_owned(),
            input_file_path: "../tmp/in".to_owned(),
            output_file_path: "../tmp/out".to_owned(),
            answer_file_path: "../tmp/ans".to_owned(),
            rlimit_config: TEST_CONFIG,
        };
        let result = run_interact(
            &runner_config,
            &String::from("../test-program/checkers/interactor-a-plus-b"),
        )
        .expect("error")
        .unwrap();
        println!("{:?}", result);
    }
}
