use std::{fs, path::PathBuf};

use anyhow::anyhow;

use crate::{
    compiler::{Compiler, Language},
    error::JudgeCoreError, utils::copy_recursively,
};

pub enum PackageType {
    ICPC,
}

pub struct JudgeBuilder {
    package_type: PackageType,
    package_path: PathBuf,
    runtime_path: PathBuf,
    src_language: Language,
    src_path: PathBuf,
    built: bool,
}

impl JudgeBuilder {
    pub fn new(
        package_type: PackageType,
        package_path: PathBuf,
        runtime_path: PathBuf,
        src_language: Language,
        src_path: PathBuf,
    ) -> Self {
        Self {
            package_type,
            package_path,
            runtime_path,
            src_language,
            src_path,
            built: false,
        }
    }

    pub fn build(&mut self) -> Result<(), JudgeCoreError> {
        match self.package_type {
            PackageType::ICPC => self.build_icpc(),
        }
    }

    fn build_icpc(&mut self) -> Result<(), JudgeCoreError> {
        fs::create_dir_all(self.runtime_path.clone())?;
        // copy checker to runtime path
        let package_output_validators_path = self.package_path.join("output_validators");
        if package_output_validators_path.exists() {
            log::warn!("Output validators found, but not supported yet");
        } else {
            log::info!("No output validators found, using default checker");
        }
        // copy testcases to runtime path
        let package_testcases_path = self.package_path.join("data");
        let runtime_testcases_path = self.runtime_path.join("data");
        if package_testcases_path.exists() {
            copy_recursively(&package_testcases_path, &runtime_testcases_path)?;
        } else {
            return Err(JudgeCoreError::AnyhowError(anyhow!(
                "Testcases not found"
            )));
        }
        if self.src_path.exists() {
            let compiler = Compiler::new(self.src_language, vec![]);
            compiler.compile(
                self.src_path.to_str().unwrap(),
                self.runtime_path.join("program").to_str().unwrap(),
            )?;
        } else {
            return Err(JudgeCoreError::AnyhowError(anyhow!(
                "Source file not found"
            )));
        }

        self.built = true;
        Ok(())
    }
}

#[cfg(test)]
pub mod builder {
    use super::{JudgeBuilder, Language, PackageType};
    use std::path::PathBuf;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_build_icpc() {
        init();
        let mut builder = JudgeBuilder::new(
            PackageType::ICPC,
            PathBuf::from("../test-collection/packages/icpc/hello_world"),
            PathBuf::from("../tmp/icpc"),
            Language::Cpp,
            PathBuf::from("../test-collection/src/programs/infinite_loop.cpp"),
        );
        match builder.build() {
            Ok(_) => {
                log::info!("Build success");
            }
            Err(e) => panic!("{:?}", e),
        }
    }
}
