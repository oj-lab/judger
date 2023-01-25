use libseccomp::error::SeccompError;
use nix::errno::Errno;
use std::ffi::NulError;
use std::io;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum JudgeCoreError {
    NixErrno(Errno),
    SeccompError(SeccompError),
    FFINulError(NulError),
    IOError(io::Error),
    FromUtf8Error(FromUtf8Error),
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

impl From<FromUtf8Error> for JudgeCoreError {
    fn from(error: FromUtf8Error) -> JudgeCoreError {
        JudgeCoreError::FromUtf8Error(error)
    }
}