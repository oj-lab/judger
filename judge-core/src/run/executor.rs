use crate::error::{path_not_exist, JudgeCoreError};
use crate::{compiler::Language, utils::get_pathbuf_str};
use nix::unistd::execve;
use serde_derive::Serialize;
use std::{convert::Infallible, ffi::CString, path::PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct Executor {
    pub language: Language,
    pub path: PathBuf,
    pub additional_args: Vec<String>,
}

// Do not do logging or some other additional things which may use system calls
// these will blocked by seccomp filter unexpectedly
impl Executor {
    pub fn new(language: Language, path: PathBuf) -> Result<Self, JudgeCoreError> {
        if !path.exists() {
            return Err(path_not_exist(&path));
        }

        Ok(Self {
            language,
            path,
            additional_args: vec![],
        })
    }

    pub fn set_additional_args(&mut self, args: Vec<String>) {
        self.additional_args = args;
    }

    pub fn exec(&self) -> Result<Infallible, JudgeCoreError> {
        let (command, args) = self.build_execute_cmd_with_args()?;
        let mut final_args = args;
        final_args.extend(self.additional_args.clone());
        let c_args = final_args
            .iter()
            .map(|s| CString::new(s.as_bytes()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(execve(
            &CString::new(command)?,
            c_args.as_slice(),
            &[CString::new("")?],
        )?)
    }

    fn build_execute_cmd_with_args(&self) -> Result<(String, Vec<String>), JudgeCoreError> {
        let path_string = get_pathbuf_str(&self.path)?;
        let command = match self.language {
            Language::Rust => &path_string,
            Language::Cpp => &path_string,
            Language::Python => "/usr/bin/python3",
        }
        .to_owned();
        let args = match self.language {
            Language::Rust => vec![],
            Language::Cpp => vec![],
            Language::Python => {
                vec![path_string]
            }
        };
        Ok((command, args))
    }
}

// Removed executor unit tests, these test should run in sandbox
