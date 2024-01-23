use std::io;

use judge_core::error::JudgeCoreError;

#[derive(Debug)]
pub enum JudgeServiceError {
    JudgeCoreError(JudgeCoreError),
    IOError(io::Error),
    RcloneError(anyhow::Error),
    AnyhowError(anyhow::Error),
}

impl From<JudgeCoreError> for JudgeServiceError {
    fn from(error: JudgeCoreError) -> JudgeServiceError {
        JudgeServiceError::JudgeCoreError(error)
    }
}

impl From<io::Error> for JudgeServiceError {
    fn from(error: io::Error) -> JudgeServiceError {
        JudgeServiceError::IOError(error)
    }
}

impl From<anyhow::Error> for JudgeServiceError {
    fn from(error: anyhow::Error) -> JudgeServiceError {
        JudgeServiceError::AnyhowError(error)
    }
}
