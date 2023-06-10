use crate::compiler::Language;
use crate::error::JudgeCoreError;
use crate::result::{check_user_result, JudgeResultInfo, check_checker_result};
use crate::run::executor::Executor;
use crate::run::process_listener::{ProcessExitMessage, ProcessListener};
use crate::run::sandbox::{RawRunResultInfo, Sandbox, SCRIPT_LIMIT_CONFIG};

use nix::errno::Errno;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::sys::epoll::{
    epoll_create1, epoll_ctl, epoll_wait, EpollCreateFlags, EpollEvent, EpollFlags, EpollOp,
};
use nix::unistd::{pipe, read, write};
use std::fs::File;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::PathBuf;

use super::JudgeConfig;

fn set_fd_non_blocking(fd: RawFd) -> Result<libc::c_int, JudgeCoreError> {
    log::debug!("Setting fd={} to non blocking", fd);
    Ok(fcntl(fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK))?)
}

/// write the content of `from` to `to`, record to output.
/// `from` will be set to non-blocking mode.
fn pump_proxy_pipe(from: RawFd, to: RawFd, output: RawFd) -> Result<(), JudgeCoreError> {
    set_fd_non_blocking(from)?;

    let mut buf = [0; 1024];
    loop {
        match read(from, &mut buf) {
            Ok(nread) => {
                log::debug!("{} read. {} -> {}", nread, from, to);
                write(to, &buf[..nread])?;
                write(output, &buf[..nread])?;
            }
            Err(e) => {
                if e == Errno::EAGAIN || e == Errno::EWOULDBLOCK {
                    return Ok(());
                }
                panic!("failed to read from pipe");
            }
        }
    }
}

/// `from` will be set to non-blocking mode.
fn read_string_from_fd(from: RawFd) -> Result<String, JudgeCoreError> {
    set_fd_non_blocking(from)?;

    let mut res_buf = Vec::new();
    let mut buf = [0; 1024];
    log::debug!("Reading from fd={}", from);
    loop {
        log::debug!("Reading from fd={}", from);
        match read(from, &mut buf) {
            Ok(nread) => {
                log::debug!("{} read. {}", nread, from);
                res_buf.extend_from_slice(&buf[..nread]);
            }
            Err(e) => {
                if e == Errno::EAGAIN || e == Errno::EWOULDBLOCK {
                    let buf_string = String::from_utf8(res_buf)?;
                    return Ok(buf_string);
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
) -> Result<Option<JudgeResultInfo>, JudgeCoreError> {
    log::debug!("Creating epoll");
    let epoll_fd = epoll_create1(EpollCreateFlags::EPOLL_CLOEXEC)?;

    log::debug!("Creating interact pipes");
    let (proxy_read_user, user_write_proxy) = pipe()?;
    let (proxy_read_interactor, interactor_write_proxy) = pipe()?;
    let (user_read_proxy, proxy_write_user) = pipe()?;
    let (interactor_read_proxy, proxy_write_interactor) = pipe()?;

    log::debug!("Adding read proxy fds to epoll");
    add_epoll_fd(epoll_fd, proxy_read_user)?;
    add_epoll_fd(epoll_fd, proxy_read_interactor)?;

    log::debug!("Creating exit report pipes with epoll");
    let (user_exit_read, user_exit_write) = pipe()?;
    let (interactor_exit_read, interactor_exit_write) = pipe()?;
    add_epoll_fd(epoll_fd, user_exit_read)?;
    add_epoll_fd(epoll_fd, interactor_exit_read)?;

    let mut user_listener = ProcessListener::new()?;
    let mut interact_listener = ProcessListener::new()?;
    user_listener.set_exit_fd(user_exit_write, 41u8);
    interact_listener.set_exit_fd(interactor_exit_write, 42u8);

    log::debug!(
        "Opening output file path={}",
        runner_config.output_file_path
    );
    if !PathBuf::from(&output_path).exists() {
        File::create(output_path)?;
    }
    let output_file = File::options()
        .write(true)
        .truncate(true) // Overwrite the whole content of this file
        .open(output_path)?;
    let output_raw_fd: RawFd = output_file.as_raw_fd();

    let user_executor = Executor::new(
        runner_config.language,
        PathBuf::from(runner_config.program_path.to_owned()),
        vec![String::from("")],
    )?;
    let mut user_sandbox = Sandbox::new(
        user_executor,
        runner_config.rlimit_configs.clone(),
        Some(user_read_proxy),
        Some(user_write_proxy),
        true,
    )?;
    user_listener.spawn_with_sandbox(&mut user_sandbox)?;

    let first_args: String = String::from("");
    let interact_args = vec![
        first_args,
        runner_config.input_file_path.to_owned(),
        runner_config.output_file_path.to_owned(),
        runner_config.answer_file_path.to_owned(),
    ];
    let interact_executor = Executor::new(
        Language::Cpp,
        PathBuf::from(interactor_path.to_string()),
        interact_args,
    )?;
    let mut interact_sandbox = Sandbox::new(
        interact_executor,
        SCRIPT_LIMIT_CONFIG,
        Some(interactor_read_proxy),
        Some(interactor_write_proxy),
        false,
    )?;
    interact_listener.spawn_with_sandbox(&mut interact_sandbox)?;

    log::debug!("Starting epoll");
    let mut events = [EpollEvent::empty(); 128];
    let mut user_exited = false;
    let mut interactor_exited = false;
    let mut option_user_result: Option<RawRunResultInfo> = None;
    loop {
        let num_events = epoll_wait(epoll_fd, &mut events, -1)?;
        log::debug!("{} events found!", num_events);

        for event in events.iter().take(num_events) {
            log::debug!("Event: {:?}", event);
            let fd = event.data() as RawFd;
            if fd == user_exit_read {
                log::debug!("{:?} user fd exited", fd);
                user_exited = true;
                let buf_string = read_string_from_fd(fd as RawFd)?;
                log::debug!("Raw Result info: {}", buf_string);
                let exit_msg: ProcessExitMessage = serde_json::from_str(&buf_string)?;
                option_user_result = exit_msg.option_run_result;
            }
            if fd == interactor_exit_read {
                log::debug!("{:?} interactor fd exited", fd);
                interactor_exited = true;
                let buf_string = read_string_from_fd(fd as RawFd)?;
                log::debug!("Raw Result info: {}", buf_string);
                let _interactor_result: ProcessExitMessage = serde_json::from_str(&buf_string)?;
            }
            if fd == proxy_read_user {
                log::debug!("proxy_read_user {} fd read", fd);
                pump_proxy_pipe(proxy_read_user, proxy_write_interactor, output_raw_fd)?;
            }
            if fd == proxy_read_interactor {
                log::debug!("proxy_read_interactor {} fd read", fd);
                pump_proxy_pipe(proxy_read_interactor, proxy_write_user, output_raw_fd)?;
            }
        }
        if user_exited && interactor_exited {
            log::debug!("Both user and interactor exited");
            break;
        }
    }
    log::debug!("Epoll finished!");

    if let Some(user_result) = option_user_result {
        let option_user_verdict = check_user_result(&user_result);
        if let Some(verdict) = option_user_verdict {
            return Ok(Some(JudgeResultInfo {
                verdict,
                time: user_result.real_time_cost,
                memory: user_result.resource_usage.max_rss,
                exit_status: user_result.exit_status,
                checker_exit_status: 0,
            }));
        }
    }

    log::debug!("Creating sandbox for checker process");
    if let Some(checker_path) = runner_config.custom_checker_path.clone() {
        let first_args = String::from("");
        let checker_args = vec![
            first_args,
            runner_config.input_file_path.to_owned(),
            runner_config.output_file_path.to_owned(),
            runner_config.answer_file_path.to_owned(),
            runner_config.check_file_path.to_owned(),
        ];
        let checker_executor =
            Executor::new(Language::Cpp, PathBuf::from(checker_path), checker_args)?;
        let mut checker_sandbox =
            Sandbox::new(checker_executor, SCRIPT_LIMIT_CONFIG, None, None, false)?;

        log::debug!("Spawning checker process");
        let _checker_spawn = checker_sandbox.spawn()?;
        log::debug!("Waiting for checker process");
        let checker_result = checker_sandbox.wait()?;
        let checker_verdict = check_checker_result(&checker_result);
        Ok(Some(JudgeResultInfo {
            verdict: checker_verdict,
            time: checker_result.real_time_cost,
            memory: checker_result.resource_usage.max_rss,
            exit_status: checker_result.exit_status,
            checker_exit_status: checker_result.exit_status,
        }))
    } else {
        Err(JudgeCoreError::AnyhowError(anyhow::anyhow!(
            "Checker path is not provided"
        )))
    }
}

#[cfg(test)]
pub mod interact_judge_test {
    use crate::{compiler::Language, judge::JudgeConfig, run::sandbox::RlimitConfigs};

    use super::run_interact;

    const TEST_CONFIG: RlimitConfigs = RlimitConfigs {
        stack_limit: Some((64 * 1024 * 1024, 64 * 1024 * 1024)),
        as_limit: Some((64 * 1024 * 1024, 64 * 1024 * 1024)),
        cpu_limit: Some((1, 2)),
        nproc_limit: Some((1, 1)),
        fsize_limit: Some((1024, 1024)),
    };

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn test_run_interact() {
        init();
        let runner_config = JudgeConfig {
            language: Language::Cpp,
            program_path: "./../test-collection/dist/programs/read_and_write".to_owned(),
            custom_checker_path: Some("./../test-collection/dist/checkers/lcmp".to_owned()),
            input_file_path: "../tmp/in".to_owned(),
            output_file_path: "../tmp/out".to_owned(),
            answer_file_path: "../tmp/ans".to_owned(),
            check_file_path: "../tmp/check".to_owned(),
            rlimit_configs: TEST_CONFIG,
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
