use serde_derive::{Deserialize, Serialize};
use serde_yaml;
use std::{fs, path::PathBuf, str::FromStr};

use crate::{
    compiler::{Compiler, Language},
    error::{path_not_exist, JudgeCoreError},
    judge::{CheckerConfig, ProgramConfig, RuntimeConfig, TestdataConfig},
    run::executor::Executor,
    run::sandbox::RlimitConfigs,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageType {
    ICPC,
}

impl PackageType {
    pub fn validate(&self, package_path: PathBuf) -> bool {
        match self {
            Self::ICPC => {
                if !package_path.exists() {
                    return false;
                }
                // TODO: validate package
                true
            }
        }
    }
}

impl FromStr for PackageType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "icpc" => Ok(Self::ICPC),
            _ => Err(anyhow::anyhow!("PackageType not found: {}", s)),
        }
    }
}

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

const DEFAULT_RLIMIT_CONFIGS: RlimitConfigs = RlimitConfigs {
    stack_limit: Some((64 * 1024 * 1024, 64 * 1024 * 1024)),
    as_limit: Some((64 * 1024 * 1024, 64 * 1024 * 1024)),
    cpu_limit: Some((1, 2)),
    nproc_limit: Some((1, 1)),
    fsize_limit: Some((1024, 1024)),
};

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
        // copy testcases to runtime path
        let package_testcases_path = input.package_path.join("data");
        let runtime_testcases_path = input.runtime_path.join("data");
        let testdata_configs = if package_testcases_path.exists() {
            copy_testdata_recursively(&package_testcases_path, &runtime_testcases_path)?
        } else {
            return Err(path_not_exist(&package_testcases_path));
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

        let rlimit_config = read_icpc_rlimit(&input.package_path)?;
        log::info!("rlimit read {:?}", rlimit_config);
        let runtime_config = RuntimeConfig {
            rlimit_configs: rlimit_config,
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

fn read_icpc_rlimit(src: &PathBuf) -> Result<RlimitConfigs, JudgeCoreError> {
    log::debug!("reading rlimit from {:?}", src);
    let stack_limit = DEFAULT_RLIMIT_CONFIGS.stack_limit;
    let mut as_limit = DEFAULT_RLIMIT_CONFIGS.as_limit;
    let mut cpu_limit = DEFAULT_RLIMIT_CONFIGS.cpu_limit;
    let nproc_limit = DEFAULT_RLIMIT_CONFIGS.nproc_limit;
    let mut fsize_limit = DEFAULT_RLIMIT_CONFIGS.fsize_limit;
    let time_limit_path = src.join(".timelimit");
    if time_limit_path.exists() {
        let content = fs::read_to_string(time_limit_path).unwrap();
        let time_limit = content.trim().parse::<u64>().unwrap();
        cpu_limit = Some((time_limit, time_limit));
    } else {
        log::info!("timelimit file not found, use default config");
    }
    let yaml_path = src.join("problem.yaml");
    if yaml_path.exists() {
        let content = fs::read_to_string(yaml_path).unwrap();
        let problem_meta = serde_yaml::from_str::<serde_yaml::Value>(&content).unwrap();
        if let Some(limits) = problem_meta.get("limits") {
            if let Some(memory_limit) = limits.get("memory") {
                if let Some(memory_u64) = memory_limit.as_u64() {
                    // the unit of as_limit is byte
                    // TODO: we need some comment for developer to know this
                    as_limit = Some((memory_u64 * 1024 * 1024, memory_u64 * 1024 * 1024));
                }
            }
            if let Some(output) = limits.get("output") {
                if let Some(output_u64) = output.as_u64() {
                    fsize_limit = Some((output_u64, output_u64));
                }
            }
        }
    }
    Ok(RlimitConfigs {
        stack_limit,
        as_limit,
        cpu_limit,
        nproc_limit,
        fsize_limit,
    })
}

#[cfg(test)]
pub mod builder {
    use crate::judge::{common::run_judge, JudgeConfig};

    use super::{JudgeBuilder, JudgeBuilderInput, Language, PackageType};
    use std::path::PathBuf;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

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
}
