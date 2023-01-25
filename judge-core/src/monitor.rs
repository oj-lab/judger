use crate::{
    error::JudgeCoreError,
    utils::killer::timeout_killer,
    runner::{run_process, RunnerConfig},
    utils::get_default_rusage,
};
use libc::{c_int, rusage, wait4, WEXITSTATUS, WSTOPPED, WTERMSIG};
use nix::unistd::{fork, write, ForkResult};
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

            run_process(runner_config)?;

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
    use crate::result::infer_result;
    use crate::runner::ResourceLimitConfig;

    #[test]
    fn test_run_judge() {
        let runner_config = RunnerConfig {
            program_path: "./../read_and_write".to_owned(),
            input_file_path: "../tmp/in".to_owned(),
            output_file_path: "../tmp/out".to_owned(),
            rlimit_config: ResourceLimitConfig::default(),
        };
        let result = infer_result(
            run_judge(&runner_config).expect("error").unwrap(),
            runner_config,
        );
        println!("{:?}", result);
    }
}
