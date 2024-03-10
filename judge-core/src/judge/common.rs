use crate::judge::result::{
    check_checker_result, check_user_result, get_max_mem, get_run_time, JudgeResultInfo,
};
use crate::run::SCRIPT_LIMIT_CONFIG;
use crate::utils::{compare_files, get_pathbuf_str};
use crate::{error::JudgeCoreError, run::sandbox::Sandbox};

use super::result::JudgeVerdict;
use super::JudgeConfig;

use std::fs::File;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::PathBuf;
use std::time::Duration;

fn run_user(
    config: &JudgeConfig,
) -> Result<(Option<JudgeVerdict>, Duration, i64, i32), JudgeCoreError> {
    let input_file = File::open(&config.test_data.input_file_path)?;

    if !config.program.output_file_path.exists() {
        File::create(&config.program.output_file_path)?;
    }
    let program_output_file = File::options()
        .write(true)
        .truncate(true) // Overwrite the whole content of this file
        .open(&config.program.output_file_path)?;

    let input_raw_fd: RawFd = input_file.as_raw_fd();
    let program_output_raw_fd: RawFd = program_output_file.as_raw_fd();

    let user_executor = config.program.executor.clone();
    let mut user_sandbox = Sandbox::new(
        user_executor,
        config.runtime.rlimit_configs.clone(),
        Some(input_raw_fd),
        Some(program_output_raw_fd),
        true,
    )?;

    log::debug!("Spawning user process");
    let _user_spawn = user_sandbox.spawn()?;
    log::debug!("Waiting for user process");
    let user_result = user_sandbox.wait()?;
    let user_time = get_run_time(&user_result);
    let max_mem = get_max_mem(&user_result);
    Ok((
        check_user_result(config, &user_result),
        user_time,
        max_mem,
        user_result.exit_status,
    ))
}

pub fn run_checker(config: &JudgeConfig) -> Result<(JudgeVerdict, i32), JudgeCoreError> {
    if let Some(mut checker_executor) = config.checker.executor.clone() {
        let first_args = String::from("");
        let checker_args = vec![
            first_args,
            get_pathbuf_str(&config.test_data.input_file_path)?,
            get_pathbuf_str(&config.program.output_file_path)?,
            get_pathbuf_str(&config.test_data.answer_file_path)?,
            get_pathbuf_str(&config.checker.output_file_path)?,
        ];
        checker_executor.set_additional_args(checker_args);

        let mut checker_process = Sandbox::new(
            checker_executor,
            SCRIPT_LIMIT_CONFIG.clone(),
            None,
            None,
            false,
        )?;

        log::debug!("Spawning checker process");
        let _checker_spawn = checker_process.spawn()?;
        log::debug!("Waiting for checker process");
        let checker_result = checker_process.wait()?;
        Ok((
            check_checker_result(&checker_result),
            checker_result.exit_status,
        ))
    } else {
        Err(JudgeCoreError::AnyhowError(anyhow::anyhow!(
            "Checker executor is not set"
        )))
    }
}

pub fn run_judge(config: &JudgeConfig) -> Result<JudgeResultInfo, JudgeCoreError> {
    let (user_verdict, user_time, max_mem, user_exit_status) = run_user(config)?;
    if let Some(verdict) = user_verdict {
        return Ok(JudgeResultInfo {
            verdict,
            time_usage: user_time,
            memory_usage_bytes: max_mem,
            exit_status: user_exit_status,
            checker_exit_status: 0,
        });
    }

    log::debug!("Creating sandbox for checker process");
    if let Some(_checker_executor) = config.checker.executor.clone() {
        let (verdict, checker_exit_status) = run_checker(config)?;
        Ok(JudgeResultInfo {
            verdict,
            time_usage: user_time,
            memory_usage_bytes: max_mem,
            exit_status: user_exit_status,
            checker_exit_status,
        })
    } else if compare_files(
        &PathBuf::from(&config.program.output_file_path),
        &PathBuf::from(&config.test_data.answer_file_path),
    ) {
        Ok(JudgeResultInfo {
            verdict: JudgeVerdict::Accepted,
            time_usage: user_time,
            memory_usage_bytes: max_mem,
            exit_status: user_exit_status,
            checker_exit_status: 0,
        })
    } else {
        Ok(JudgeResultInfo {
            verdict: JudgeVerdict::WrongAnswer,
            time_usage: user_time,
            memory_usage_bytes: max_mem,
            exit_status: user_exit_status,
            checker_exit_status: 0,
        })
    }
}
