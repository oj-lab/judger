use libseccomp::error::SeccompError;
use nix::errno::Errno;
use std::ffi::NulError;
use std::io;
use std::path::PathBuf;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum JudgeCoreError {
    NixErrno(Errno),
    SeccompError(SeccompError),
    FFINulError(NulError),
    IOError(io::Error),
    SerdeJsonError(serde_json::Error),
    AnyhowError(anyhow::Error),
    FromUtf8Error(FromUtf8Error),
    CompileError(String),
}

impl From<Errno> for JudgeCoreError {
    fn from(error: Errno) -> JudgeCoreError {
        JudgeCoreError::NixErrno(error)
    }
}

impl From<SeccompError> for JudgeCoreError {
    fn from(error: SeccompError) -> JudgeCoreError {
        JudgeCoreError::SeccompError(error)
    }
}

impl From<NulError> for JudgeCoreError {
    fn from(error: NulError) -> JudgeCoreError {
        JudgeCoreError::FFINulError(error)
    }
}

impl From<io::Error> for JudgeCoreError {
    fn from(error: io::Error) -> JudgeCoreError {
        JudgeCoreError::IOError(error)
    }
}

impl From<anyhow::Error> for JudgeCoreError {
    fn from(error: anyhow::Error) -> JudgeCoreError {
        JudgeCoreError::AnyhowError(error)
    }
}

impl From<serde_json::Error> for JudgeCoreError {
    fn from(error: serde_json::Error) -> JudgeCoreError {
        JudgeCoreError::SerdeJsonError(error)
    }
}

impl From<FromUtf8Error> for JudgeCoreError {
    fn from(error: FromUtf8Error) -> JudgeCoreError {
        JudgeCoreError::FromUtf8Error(error)
    }
}

pub fn path_not_exist(path: &PathBuf) -> JudgeCoreError {
    JudgeCoreError::AnyhowError(anyhow::anyhow!("Path not exist: {:?}", path))
}
