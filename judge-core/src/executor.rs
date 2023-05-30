use crate::error::JudgeCoreError;
use crate::{compiler::Language, error::anyhow_error_msg};
use nix::unistd::execve;
use std::{convert::Infallible, ffi::CString, path::PathBuf};

#[derive(Clone)]
pub struct Executor {
    pub language: Language,
    pub path: PathBuf,
    pub additional_args: Vec<String>,
}

impl Executor {
    pub fn new(
        language: Language,
        path: PathBuf,
        additional_args: Vec<String>,
    ) -> Result<Self, JudgeCoreError> {
        if !path.exists() {
            return Err(anyhow_error_msg(&format!(
                "Program path not found: {:?}",
                path
            )));
        }

        Ok(Self {
            language,
            path,
            additional_args,
        })
    }

    pub fn exec(&self) -> Result<Infallible, JudgeCoreError> {
        let (command, args) = self.build_cmd_args()?;
        let mut final_args = args;
        final_args.extend(self.additional_args.clone());
        let c_args = args
            .iter()
            .map(|s| CString::new(s.as_bytes()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(execve(
            &CString::new(command)?,
            c_args.as_slice(),
            &[CString::new("")?],
        )?)
    }

    fn build_cmd_args(&self) -> Result<(String, Vec<String>), JudgeCoreError> {
        let path_string = match self.path.clone().to_str() {
            Some(path_string) => Ok(path_string.to_owned()),
            None => Err(anyhow_error_msg(
                "excutor did not find path for this language",
            )),
        };
        let command = match self.language {
            Language::Rust => path_string,
            Language::Cpp => path_string,
            Language::Python => Ok("/usr/bin/python3".to_owned()),
        };
        let args = match self.language {
            Language::Rust => vec![],
            Language::Cpp => vec![],
            Language::Python => {
                vec![path_string?]
            }
        };
        Ok((command?, args))
    }
}

#[cfg(test)]
pub mod executor {
    use std::path::PathBuf;

    use super::Executor;
    use crate::compiler::Language;

    #[test]
    fn test_exec_python() {
        let executor = Executor::new(
            Language::Python,
            PathBuf::from("../test-collection/src/programs/read_and_write.py"),
            vec![String::from("")],
        )
        .expect("executor init failed");
        match executor.exec() {
            Ok(result) => {
                log::debug!("{:?}", result);
            }
            Err(e) => {
                log::error!("meet error: {:?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn test_exec_cpp() {
        let executor = Executor::new(
            Language::Cpp,
            PathBuf::from("../test-collection/dist/programs/memory_limit".to_string()),
            vec![String::from("")],
        ).expect("executor init failed");
        match executor.exec() {
            Ok(result) => {
                log::debug!("{:?}", result);
            }
            Err(e) => {
                log::error!("meet error: {:?}", e);
                assert!(false);
            }
        }
    }
}
