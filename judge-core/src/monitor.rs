use crate::sandbox::{ResourceLimitConfig, SandBox};
use crate::{error::JudgeCoreError, killer::timeout_killer, utils::get_default_rusage};
use libc::{c_int, rusage, wait4, WEXITSTATUS, WSTOPPED, WTERMSIG};
use nix::unistd::{fork, write, ForkResult};
use std::fs::File;
use std::os::unix::io::{AsRawFd, RawFd};
use std::{
    thread,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct RawJudgeResultInfo {
    pub exit_status: c_int,
    pub exit_signal: c_int,
    pub exit_code: c_int,
    pub real_time_cost: Duration,
    pub resource_usage: rusage,
}

pub struct RunnerConfig {
    pub program_path: String,
    pub checker_path: String,
    pub input_file_path: String,
    pub output_file_path: String,
    pub answer_file_path: String,
    pub rlimit_config: ResourceLimitConfig,
}

pub fn run_judge(
    runner_config: &RunnerConfig,
) -> Result<Option<RawJudgeResultInfo>, JudgeCoreError> {
    let now = Instant::now();

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            println!(
                "Continuing execution in parent process, new child has pid: {}",
                child
            );

            thread::spawn(move || timeout_killer(child.as_raw() as u32, 5000));
            println!("timeout_killer has been set");

            let mut status: c_int = 0;
            let mut usage: rusage = get_default_rusage();
            unsafe {
                wait4(child.as_raw() as i32, &mut status, WSTOPPED, &mut usage);
            }

            println!("Detected process exit");

            Ok(Some(RawJudgeResultInfo {
                exit_status: status,
                exit_signal: WTERMSIG(status),
                exit_code: WEXITSTATUS(status),
                real_time_cost: now.elapsed(),
                resource_usage: usage,
            }))
        }
        Ok(ForkResult::Child) => {
            // Unsafe to use `println!` (or `unwrap`) here. See Safety.
            write(libc::STDOUT_FILENO, "I'm a new child process\n".as_bytes()).ok();

            let sandbox = SandBox::new().unwrap();
            let input_file = File::open(&runner_config.input_file_path)?;
            let output_file = File::options()
                .write(true)
                .truncate(true) // Overwrite the whole content of this file
                .open(&runner_config.output_file_path)
                .unwrap();
            let input_raw_fd: RawFd = input_file.as_raw_fd();
            let output_raw_fd: RawFd = output_file.as_raw_fd();
            sandbox.set_io(input_raw_fd, output_raw_fd);
            sandbox.set_limit(&runner_config.rlimit_config)?;
            sandbox.exec(&runner_config.program_path).unwrap();

            Ok(None)
        }
        Err(_) => {
            println!("Fork failed");

            Ok(None)
        }
    }
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
            checker_path: "./../test-program/checkers/ncmp".to_owned(),
            input_file_path: "../tmp/in".to_owned(),
            output_file_path: "../tmp/out".to_owned(),
            answer_file_path: "../tmp/ans".to_owned(),
            rlimit_config: TEST_CONFIG,
        };
        let result = run_judge(&runner_config).expect("error").unwrap();
        println!("{:?}", result);
    }
}
