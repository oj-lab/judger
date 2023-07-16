use std::{fs, path::PathBuf};

use crate::{
    compiler::{Compiler, Language},
    error::{path_not_exist, JudgeCoreError},
    judge::{CheckerConfig, ProgramConfig, TestdataConfig},
    run::executor::Executor,
};

pub enum PackageType {
    ICPC,
}

pub enum JudgeType {
    COMMON,
    INTERACT,
}

pub struct JudgeBuilder {
    pub judge_type: JudgeType,
    pub testdata_configs: Vec<TestdataConfig>,
    pub program_config: ProgramConfig,
    pub checker_config: CheckerConfig,
}

pub struct JudgeBuilderInput {
    pub package_type: PackageType,
    pub package_path: PathBuf,
    pub runtime_path: PathBuf,
    pub src_language: Language,
    pub src_path: PathBuf,
}

impl JudgeBuilder {
    pub fn new(input: JudgeBuilderInput) -> Result<Self, JudgeCoreError> {
        Self::build(input)
    }

    fn build(input: JudgeBuilderInput) -> Result<Self, JudgeCoreError> {
        match input.package_type {
            PackageType::ICPC => Ok(Self::build_icpc(input)?),
        }
    }

    // TODO: Load Problem's run config from config
    fn build_icpc(input: JudgeBuilderInput) -> Result<Self, JudgeCoreError> {
        fs::create_dir_all(input.runtime_path.clone())?;
        // copy checker to runtime path
        let package_output_validators_path = input.package_path.join("output_validators");
        if package_output_validators_path.exists() {
            log::warn!("Output validators found, but not supported yet");
        } else {
            log::info!("No output validators found, using default checker");
        }
        let checker_config = CheckerConfig {
            executor: None,
            output_file_path: input.runtime_path.join("checker.out"),
        };
        let testdata_configs: Vec<TestdataConfig>;
        // copy testcases to runtime path
        let package_testcases_path = input.package_path.join("data");
        let runtime_testcases_path = input.runtime_path.join("data");
        if package_testcases_path.exists() {
            testdata_configs =
                copy_testdata_recursively(&package_testcases_path, &runtime_testcases_path)?;
        } else {
            return Err(path_not_exist(&package_testcases_path));
        }

        let program_config: ProgramConfig;
        if input.src_path.exists() {
            let compiler = Compiler::new(input.src_language, vec![]);
            compiler.compile(&input.src_path, &input.runtime_path.join("program"))?;
            program_config = ProgramConfig {
                executor: Executor::new(input.src_language, input.runtime_path.join("program"))?,
                output_file_path: input.runtime_path.join("program.out"),
            };
        } else {
            return Err(path_not_exist(&input.src_path));
        }

        Ok(Self {
            judge_type: JudgeType::COMMON,
            testdata_configs,
            program_config,
            checker_config,
        })
    }
}

fn copy_testdata_recursively(
    src: &PathBuf,
    dest: &PathBuf,
) -> Result<Vec<TestdataConfig>, JudgeCoreError> {
    log::debug!("copying {:?} to {:?}", src, dest);
    let mut testdata_configs: Vec<TestdataConfig> = vec![];
    if fs::metadata(src)?.is_file() {
        if src.extension() == Some(std::ffi::OsStr::new("in")) {
            log::info!("founding testdata pair for: {:?}", src);
            let answer_path = src.with_extension("ans");
            if answer_path.exists() {
                fs::copy(src, dest)?;
                testdata_configs.push(TestdataConfig {
                    input_file_path: src.clone(),
                    answer_file_path: answer_path,
                });
            }
        }
    } else {
        if !dest.exists() || !fs::metadata(dest)?.is_dir() {
            log::debug!("creating dir: {:?}", dest);
            fs::create_dir_all(dest)?;
        }
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let file_name = src_path.file_name().unwrap();
            let dest_path = dest.join(file_name);
            testdata_configs.append(&mut copy_testdata_recursively(&src_path, &dest_path)?);
        }
    }

    Ok(testdata_configs)
}

#[cfg(test)]
pub mod builder {
    use crate::{judge::{JudgeConfig, RuntimeConfig, common::run_judge}, run::sandbox::RlimitConfigs};

    use super::{JudgeBuilder, JudgeBuilderInput, Language, PackageType};
    use std::path::PathBuf;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    const TEST_CONFIG: RlimitConfigs = RlimitConfigs {
        stack_limit: Some((64 * 1024 * 1024, 64 * 1024 * 1024)),
        as_limit: Some((64 * 1024 * 1024, 64 * 1024 * 1024)),
        cpu_limit: Some((1, 2)),
        nproc_limit: Some((1, 1)),
        fsize_limit: Some((1024, 1024)),
    };

    #[test]
    fn test_build_icpc() {
        init();
        let builder = JudgeBuilder::new(JudgeBuilderInput {
            package_type: PackageType::ICPC,
            package_path: PathBuf::from("../test-collection/packages/icpc/hello_world"),
            runtime_path: PathBuf::from("../tmp/icpc"),
            src_language: Language::Cpp,
            src_path: PathBuf::from("../test-collection/src/programs/read_and_write.cpp"),
        })
        .unwrap();
        log::info!("builder has {} testdata configs", builder.testdata_configs.len());
        for idx in 0..builder.testdata_configs.len() {
            log::info!("runing testdata {}", idx);
            let judge_config = JudgeConfig {
                test_data: builder.testdata_configs[idx].clone(),
                program: builder.program_config.clone(),
                checker: builder.checker_config.clone(),
                runtime: RuntimeConfig {
                    rlimit_configs: TEST_CONFIG,
                }
            };

            let res = run_judge(&judge_config);
            match res {
                Ok(info) => log::info!("{:?}", info),
                Err(e) => panic!("{:?}", e)
            }
        }
    }
}
