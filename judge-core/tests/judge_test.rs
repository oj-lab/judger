use std::path::PathBuf;

use judge_core::{
    compiler::Language,
    judge::{
        builder::{JudgeBuilder, JudgeBuilderInput},
        interact::run_interact,
        result::JudgeVerdict,
        CheckerConfig, JudgeConfig, ProgramConfig, RuntimeConfig, TestdataConfig,
    },
    package::PackageType,
    run::executor::Executor,
    sandbox::RlimitConfigs,
};

use judge_core::judge::common::run_judge;

const TEST_DATA_PATH: &str = "tests/data";
const TEST_TEMP_PATH: &str = "tests/temp";

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

fn build_test_config(program_executor: Executor) -> JudgeConfig {
    JudgeConfig {
        runtime: RuntimeConfig {
            rlimit_configs: TEST_CONFIG,
        },
        test_data: TestdataConfig {
            input_file_path: PathBuf::from(TEST_DATA_PATH)
                .join("packages/icpc/hello_world/data/secret/0.in"),
            answer_file_path: PathBuf::from(TEST_DATA_PATH)
                .join("packages/icpc/hello_world/data/secret/0.ans"),
        },
        checker: CheckerConfig {
            executor: None,
            output_file_path: PathBuf::from(TEST_TEMP_PATH).join("checker.out"),
        },
        program: ProgramConfig {
            executor: program_executor,
            output_file_path: PathBuf::from(TEST_TEMP_PATH).join("program.out"),
        },
    }
}

#[test]
fn test_run_judge() {
    init();
    log::debug!("current dir: {:?}", std::env::current_dir().unwrap());

    let program_path =
        PathBuf::from(TEST_DATA_PATH).join("built-in-programs/build/src/programs/read_and_write");
    let program_executor = Executor::new(Language::Cpp, program_path).unwrap();

    let runner_config = build_test_config(program_executor);
    let result = run_judge(&runner_config);
    if let Ok(result) = result {
        log::debug!("{:?}", result);
        assert_eq!(result.verdict, JudgeVerdict::Accepted);
    } else {
        log::debug!("{:?}", result);
        unreachable!()
    }
}

#[test]
fn test_run_judge_python() {
    init();
    log::debug!("current dir: {:?}", std::env::current_dir().unwrap());

    let program_path =
        PathBuf::from(TEST_DATA_PATH).join("built-in-programs/src/programs/read_and_write.py");
    let program_executor = Executor::new(Language::Python, program_path.clone()).unwrap();

    let runner_config = build_test_config(program_executor);
    let result = run_judge(&runner_config);
    if let Ok(result) = result {
        log::debug!("{:?}", result);
        assert_eq!(result.verdict, JudgeVerdict::Accepted);
    } else {
        log::debug!("{:?}", result);
        unreachable!()
    }
}

#[test]
fn test_run_tle() {
    init();
    let program_path =
        PathBuf::from(TEST_DATA_PATH).join("built-in-programs/build/src/programs/infinite_loop");
    let program_executor = Executor::new(Language::Cpp, program_path).unwrap();

    let runner_config = build_test_config(program_executor);
    let result = run_judge(&runner_config);
    assert!(result.is_ok());
    if let Ok(result) = result {
        log::debug!("{:?}", result);
        assert_eq!(result.verdict, JudgeVerdict::TimeLimitExceeded);
    }
}

#[test]
fn test_run_mle() {
    init();
    let program_path =
        PathBuf::from(TEST_DATA_PATH).join("built-in-programs/build/src/programs/memory_limit");
    let program_executor = Executor::new(Language::Cpp, program_path).unwrap();

    let runner_config = build_test_config(program_executor);
    let result = run_judge(&runner_config);
    assert!(result.is_ok());
    if let Ok(result) = result {
        log::debug!("{:?}", result);
        assert_eq!(result.verdict, JudgeVerdict::RuntimeError);
    }
}

#[test]
fn test_run_interact() {
    init();
    let interactor_executor = Executor::new(
        Language::Cpp,
        PathBuf::from(TEST_DATA_PATH).join("built-in-programs/build/src/checkers/interactor-echo"),
    )
    .unwrap();
    let program_executor = Executor::new(
        Language::Cpp,
        PathBuf::from(TEST_DATA_PATH).join("built-in-programs/build/src/programs/read_and_write"),
    )
    .unwrap();
    let runner_config = JudgeConfig {
        checker: CheckerConfig {
            executor: Some(
                Executor::new(
                    Language::Cpp,
                    PathBuf::from(TEST_DATA_PATH).join("built-in-programs/build/src/checkers/lcmp"),
                )
                .unwrap(),
            ),
            output_file_path: PathBuf::from(TEST_TEMP_PATH).join("checker.out"),
        },
        ..build_test_config(program_executor)
    };
    let result = run_interact(
        &runner_config,
        interactor_executor,
        &PathBuf::from(TEST_TEMP_PATH).join("interact.out"),
    );
    match result {
        Ok(Some(result)) => {
            log::debug!("{:?}", result);
            assert!(result.verdict == JudgeVerdict::Accepted);
        }
        Ok(None) => {
            log::debug!("Ignoring this result, for it's from a fork child process");
        }
        Err(e) => {
            log::error!("meet error: {:?}", e);
            unreachable!()
        }
    }
}

#[test]
fn test_build_icpc() {
    init();
    let builder = JudgeBuilder::new(JudgeBuilderInput {
        package_type: PackageType::ICPC,
        package_path: PathBuf::from(TEST_DATA_PATH).join("packages/icpc/hello_world"),
        runtime_path: PathBuf::from(TEST_TEMP_PATH).join("hello_world"),
        src_language: Language::Cpp,
        src_path: PathBuf::from(TEST_DATA_PATH)
            .join("built-in-programs/src/programs/read_and_write.cpp"),
    })
    .unwrap();
    log::info!("builder: {:?}", builder);
    for idx in 0..builder.testdata_configs.len() {
        log::info!("runing testdata {}", idx);
        let judge_config = JudgeConfig {
            test_data: builder.testdata_configs[idx].clone(),
            program: builder.program_config.clone(),
            checker: builder.checker_config.clone(),
            runtime: builder.runtime_config.clone(),
        };

        let res = run_judge(&judge_config);
        match res {
            Ok(info) => log::info!("{:?}", info),
            Err(e) => panic!("{:?}", e),
        }
    }
}
