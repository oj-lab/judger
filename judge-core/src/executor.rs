use crate::compiler::Language;
use crate::error::JudgeCoreError;
use nix::unistd::execve;
use std::{convert::Infallible, ffi::CString};

#[derive(Clone)]
pub struct Executor {
    pub language: Language,
    pub src_path: String,
    pub args: Vec<String>,
}

impl Executor {
    pub fn new(language: Language, src_path: String, args: Vec<String>) -> Self {
        Self {
            language,
            src_path,
            args,
        }
    }

    pub fn exec(&self) -> Result<Infallible, JudgeCoreError> {
        let command = match self.language {
            Language::Rust => self.src_path.clone(),
            Language::Cpp => self.src_path.clone(),
            Language::Python => "/usr/bin/python3".to_string(),
        };
        log::debug!("Exec command: {}", command);
        log::debug!("Preparing args for execve");
        let mut args = self.args.clone();
        if self.language == Language::Python {
            args.insert(1, self.src_path.clone());
        }
        let c_args = args
            .iter()
            .map(|s| CString::new(s.as_bytes()))
            .collect::<Result<Vec<_>, _>>()?;
        log::debug!("Running execve with c_args={:?}", c_args);
        Ok(execve(
            &CString::new(command)?,
            c_args.as_slice(),
            &[CString::new("")?],
        )?)
    }
}

#[cfg(test)]
pub mod executor {
    use super::Executor;
    use crate::compiler::Language;

    #[test]
    fn test_exec_python() {
        let executor = Executor::new(
            Language::Python,
            "../test-collection/src/programs/read_and_write.py".to_string(),
            vec![String::from("")],
        );
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
            "../test-collection/dist/programs/memory_limit".to_string(),
            vec![String::from("")],
        );
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
