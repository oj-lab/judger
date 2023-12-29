use serde_derive::Serialize;

use std::{fs, path::PathBuf};

use crate::{
    compiler::{Compiler, Language},
    error::{path_not_exist, JudgeCoreError},
    judge::{CheckerConfig, ProgramConfig, RuntimeConfig, TestdataConfig},
    package::PackageType,
    run::executor::Executor,
};

#[derive(Debug, Clone, Serialize)]
pub enum JudgeType {
    COMMON,
    INTERACT,
}

#[derive(Debug, Clone, Serialize)]
pub struct JudgeBuilder {
    pub judge_type: JudgeType,
    pub testdata_configs: Vec<TestdataConfig>,
    pub program_config: ProgramConfig,
    pub checker_config: CheckerConfig,
    pub runtime_config: RuntimeConfig,
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
        let package_agent = input
            .package_type
            .get_package_agent(input.package_path.clone())?;

        fs::create_dir_all(input.runtime_path.clone())?;

        let checker_config = package_agent.load_checker(input.runtime_path.join("checker.out"))?;

        // copy testcases to runtime path
        let runtime_testcases_path = input.runtime_path.join("data");
        let testdata_configs = package_agent.load_testdata(runtime_testcases_path)?;

        let rlimit_config = package_agent.get_rlimit_configs()?;
        log::info!("rlimit read {:?}", rlimit_config);
        let runtime_config = RuntimeConfig {
            rlimit_configs: rlimit_config,
        };

        let program_config = if input.src_path.exists() {
            let compiler = Compiler::new(input.src_language, vec![]);
            compiler.compile(&input.src_path, &input.runtime_path.join("program"))?;
            ProgramConfig {
                executor: Executor::new(input.src_language, input.runtime_path.join("program"))?,
                output_file_path: input.runtime_path.join("program.out"),
            }
        } else {
            return Err(path_not_exist(&input.src_path));
        };

        Ok(Self {
            judge_type: JudgeType::COMMON,
            testdata_configs,
            program_config,
            checker_config,
            runtime_config,
        })
    }
}
