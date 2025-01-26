use std::{fs, path::PathBuf};

use anyhow::anyhow;

use crate::{
    error::JudgeCoreError,
    judge::{CheckerConfig, TestdataConfig},
    sandbox::{RlimitConfigs, DEFAULT_RLIMIT_CONFIGS},
};

use super::PackageAgent;

pub struct ICPCPackageAgent {
    package_path: PathBuf,
}

impl PackageAgent for ICPCPackageAgent {
    fn init(package_path: PathBuf) -> Result<ICPCPackageAgent, JudgeCoreError> {
        if !package_path.exists() || package_path.is_file() {
            return Err(JudgeCoreError::AnyhowError(anyhow!("invalid package_path")));
        }
        Ok(Self { package_path })
    }

    fn validate(&self) -> bool {
        if !self.package_path.join("problem.yaml").exists()
            || !self.package_path.join("problem.yaml").is_file()
        {
            return false;
        }
        if !self.package_path.join("data").exists() || !self.package_path.join("data").is_dir() {
            return false;
        }

        true
    }

    fn get_rlimit_configs(&self) -> Result<RlimitConfigs, JudgeCoreError> {
        let stack_limit = DEFAULT_RLIMIT_CONFIGS.stack_limit;
        let mut as_limit = DEFAULT_RLIMIT_CONFIGS.as_limit;
        let mut cpu_limit = DEFAULT_RLIMIT_CONFIGS.cpu_limit;
        let nproc_limit = DEFAULT_RLIMIT_CONFIGS.nproc_limit;
        let mut fsize_limit = DEFAULT_RLIMIT_CONFIGS.fsize_limit;
        log::debug!("reading rlimit from {:?}", self.package_path);

        let time_limit_path = self.package_path.join(".timelimit");
        if time_limit_path.exists() {
            let content = fs::read_to_string(time_limit_path).unwrap();
            let time_limit = content.trim().parse::<u64>().unwrap();
            cpu_limit = Some((time_limit, time_limit));
        } else {
            log::info!("timelimit file not found, use default config");
        }

        let yaml_path = self.package_path.join("problem.yaml");
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

    fn load_testdata(
        &self,
        dest: PathBuf,
    ) -> Result<Vec<crate::judge::TestdataConfig>, JudgeCoreError> {
        let testdata_path = self.package_path.join("data");
        if !testdata_path.exists() || !testdata_path.is_dir() {
            return Err(JudgeCoreError::AnyhowError(anyhow!(
                "testdata dir not found"
            )));
        }

        copy_testdata_recursively(&testdata_path, &dest)
    }

    fn load_checker(&self, checker_output_path: PathBuf) -> Result<CheckerConfig, JudgeCoreError> {
        let output_validators_path = self.package_path.join("output_validators");
        if output_validators_path.exists() {
            log::warn!("Output validators found, but not supported yet");
        } else {
            log::info!("No output validators found, using default checker");
        }

        Ok(CheckerConfig {
            executor: None,
            output_file_path: checker_output_path,
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
