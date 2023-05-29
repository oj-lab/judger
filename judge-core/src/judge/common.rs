use crate::compiler::Language;
use crate::result::{
    check_checker_result, check_user_result, get_max_mem, get_run_time, JudgeVerdict,
};
use crate::sandbox::SCRIPT_LIMIT_CONFIG;
use crate::utils::compare_files;
use crate::{error::JudgeCoreError, executor::Executor, result::JudgeResultInfo, sandbox::SandBox};

use super::JudgeConfig;

use std::fs::File;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::PathBuf;

pub fn run_judge(runner_config: &JudgeConfig) -> Result<Option<JudgeResultInfo>, JudgeCoreError> {
    log::debug!("Creating sandbox for user process");
    let mut user_process = SandBox::new(true)?;
    log::debug!("Opening input file path={}", runner_config.input_file_path);
    let input_file = File::open(&runner_config.input_file_path)?;
    log::debug!(
        "Opening output file path={}",
        runner_config.output_file_path
    );
    let output_file = File::options()
        .write(true)
        .truncate(true) // Overwrite the whole content of this file
        .open(&runner_config.output_file_path)?;
    let input_raw_fd: RawFd = input_file.as_raw_fd();
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
        input_raw_fd,
        output_raw_fd,
    )?;
    if user_spawn.is_none() {
        return Ok(None);
    }
    log::debug!("Waiting for user process");
    let user_result = user_process.wait()?;
    let user_time = get_run_time(&user_result);
    let max_mem = get_max_mem(&user_result);
    if let Some(verdict) = check_user_result(&user_result) {
        return Ok(Some(JudgeResultInfo {
            verdict,
            time: user_time,
            memory: max_mem,
            exit_status: user_result.exit_status,
            checker_exit_status: 0,
        }));
    }

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
        let verdict = check_checker_result(&checker_result);
        Ok(Some(JudgeResultInfo {
            verdict,
            time: user_time,
            memory: max_mem,
            exit_status: user_result.exit_status,
            checker_exit_status: checker_result.exit_status,
        }))
    } else if compare_files(
        &PathBuf::from(&runner_config.output_file_path),
        &PathBuf::from(&runner_config.answer_file_path),
    ) {
        Ok(Some(JudgeResultInfo {
            verdict: JudgeVerdict::Accepted,
            time: user_time,
            memory: max_mem,
            exit_status: user_result.exit_status,
            checker_exit_status: 0,
        }))
    } else {
        Ok(Some(JudgeResultInfo {
            verdict: JudgeVerdict::WrongAnswer,
            time: user_time,
            memory: max_mem,
            exit_status: user_result.exit_status,
            checker_exit_status: 0,
        }))
    }
}

#[cfg(test)]
pub mod common_judge_tests {
    use crate::{
        compiler::Language, judge::JudgeConfig, result::JudgeVerdict, sandbox::ResourceLimitConfig,
    };

    use super::run_judge;

    const TEST_CONFIG: ResourceLimitConfig = ResourceLimitConfig {
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
    fn test_run_judge() {
        init();
        let runner_config = JudgeConfig {
            language: Language::Cpp,
            program_path: "./../test-collection/dist/programs/read_and_write".to_owned(),
            custom_checker_path: None,
            input_file_path: "../tmp/in".to_owned(),
            output_file_path: "../tmp/out".to_owned(),
            answer_file_path: "../tmp/ans".to_owned(),
            check_file_path: "../tmp/check".to_owned(),
            rlimit_config: TEST_CONFIG,
        };
        let result = run_judge(&runner_config);
        if let Ok(Some(result)) = result {
            log::debug!("{:?}", result);
            assert_eq!(result.verdict, JudgeVerdict::Accepted);
        } else {
            log::debug!("{:?}", result);
            assert!(false)
        }
    }

    #[test]
    fn test_run_tle() {
        let runner_config = JudgeConfig {
            language: Language::Cpp,
            program_path: "./../test-collection/dist/programs/infinite_loop".to_owned(),
            custom_checker_path: Some("./../test-collection/dist/checkers/lcmp".to_owned()),
            input_file_path: "../tmp/in".to_owned(),
            output_file_path: "../tmp/out".to_owned(),
            answer_file_path: "../tmp/ans".to_owned(),
            check_file_path: "../tmp/check".to_owned(),
            rlimit_config: TEST_CONFIG,
        };
        let result = run_judge(&runner_config);
        assert!(result.is_ok());
        if let Ok(Some(result)) = result {
            log::debug!("{:?}", result);
            assert_eq!(result.verdict, JudgeVerdict::TimeLimitExceeded);
        }
    }

    #[test]
    fn test_run_mle() {
        let runner_config = JudgeConfig {
            language: Language::Cpp,
            program_path: "./../test-collection/dist/programs/memory_limit".to_owned(),
            custom_checker_path: Some("./../test-collection/dist/checkers/lcmp".to_owned()),
            input_file_path: "../tmp/in".to_owned(),
            output_file_path: "../tmp/out".to_owned(),
            answer_file_path: "../tmp/ans".to_owned(),
            check_file_path: "../tmp/check".to_owned(),
            rlimit_config: TEST_CONFIG,
        };
        let result = run_judge(&runner_config);
        assert!(result.is_ok());
        if let Ok(Some(result)) = result {
            log::debug!("{:?}", result);
            assert_eq!(result.verdict, JudgeVerdict::RuntimeError);
        }
    }
}
