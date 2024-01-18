use judge_core::error::JudgeCoreError;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Internal Error: {0}")]
    InternalError(anyhow::Error),
    #[error("Pick Error: {0}")]
    PickFail(anyhow::Error),
    #[error("Report Error: {0}")]
    ReportFail(anyhow::Error),
    #[error("Reqwest Error: {0}")]
    ReqwestError(reqwest::Error),
    #[error("Judge Core Error")]
    JudgeError(JudgeCoreError),
}

impl From<reqwest::Error> for ClientError {
    fn from(err: reqwest::Error) -> ClientError {
        ClientError::ReqwestError(err)
    }
}

impl From<JudgeCoreError> for ClientError {
    fn from(err: JudgeCoreError) -> ClientError {
        ClientError::JudgeError(err)
    }
}
