use crate::error::JudgeCoreError;
use crate::sandbox::{RawRunResultInfo, ResourceLimitConfig, SandBox};
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
    match user_process.spawn_with_io(
        &runner_config.program_path,
        &vec![&String::from("")],
        &runner_config.rlimit_config,
        input_raw_fd,
        output_raw_fd,
    ) {
        Ok(Some((user_begin, user_pid))) => {
            let user_result = user_process.wait(user_begin, user_pid)?;

            let checker_process = SandBox::new(false)?;
            let first_args = String::from("");
            let checker_args = vec![
                &first_args,
                &runner_config.input_file_path,
                &runner_config.output_file_path,
                &runner_config.answer_file_path,
            ];
            match checker_process.spawn(
                &runner_config.checker_path,
                &checker_args,
                &runner_config.rlimit_config,
            ) {
                Ok(Some((check_begin, checker_pid))) => {
                    let checker_result = checker_process.wait(check_begin, checker_pid)?;
                    Ok(checker_result)
                }
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            }
        }
        Ok(None) => Ok(None),
        Err(e) => Err(e),
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
            checker_path: "./../test-program/checkers/lcmp".to_owned(),
            input_file_path: "../tmp/in".to_owned(),
            output_file_path: "../tmp/out".to_owned(),
            answer_file_path: "../tmp/ans".to_owned(),
            rlimit_config: TEST_CONFIG,
        };
        let result = run_judge(&runner_config).expect("error").unwrap();
        println!("{:?}", result);
    }
}
