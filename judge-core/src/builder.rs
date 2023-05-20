use std::{path::PathBuf, fs};

use anyhow::anyhow;

use crate::{compiler::{Compiler, Language}, error::JudgeCoreError};

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
            PackageType::ICPC => {
                self.build_icpc()
            }
        }
    }

    fn build_icpc(&mut self) -> Result<(), JudgeCoreError> {
        fs::create_dir_all(self.runtime_path.clone())?;
        // copy checker to runtime path
        let checker_exe_path = self.package_path.join("output_validators");
        if checker_exe_path.exists() {
            fs::create_dir_all(self.runtime_path.join("checkers"))?;
            for entry in fs::read_dir(checker_exe_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let filename = path.file_name().unwrap();
                    fs::copy(&path, self.runtime_path.join("checkers").join(filename))?;
                }
            }
        }
        // copy testcases to runtime path
        let secret_testcases_path = self.package_path.join("data/secret");
        fs::create_dir_all(self.runtime_path.join("data/secret"))?;
        for entry in fs::read_dir(secret_testcases_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                fs::copy(&path, self.runtime_path.join("data/secret").join(path.file_name().unwrap()))?;
            }
        }
        if self.src_path.exists() {
            let compiler = Compiler::new(self.src_language.clone(), vec![]);
            compiler.compile(&self.src_path.to_str().unwrap(), &self.runtime_path.join("program").to_str().unwrap())?;
        } else {
            return Err(JudgeCoreError::AnyhowError(anyhow!("Source file not found")));
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