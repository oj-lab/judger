#![allow(dead_code)]

use actix_web::{HttpResponse, ResponseError};
use judge_core::error::JudgeCoreError;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Internal Server Error: {0}")]
    InternalError(anyhow::Error),
    #[error("Internal Server Error: {0}, Msg: {1}")]
    InternalErrorWithMsg(anyhow::Error, String),

    #[error("BadRequest: {0}, Msg: {1}")]
    BadRequestWithMsg(anyhow::Error, String),

    #[error("Unauthorized: {0}")]
    Unauthorized(anyhow::Error),
    #[error("Unauthorized: {0}, Msg: {1}")]
    UnauthorizedWithMsg(anyhow::Error, String),
}

#[derive(Serialize)]
struct ServiceErrorBody {
    msg: Option<String>
}

// impl ResponseError trait allows to convert our errors into http responses with appropriate data
impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InternalError(ref err) => {
                let response_body = ServiceErrorBody {
                    msg: Some(format!("Internal Error: {}", err))
                };
                HttpResponse::InternalServerError().json(response_body)
            }
            ServiceError::InternalErrorWithMsg(ref err, ref msg) => {
                let response_body = ServiceErrorBody {
                    msg: Some(format!("Internal Error: {}, Msg: {}", err, msg))
                };
                HttpResponse::InternalServerError().json(response_body)
            }
            ServiceError::BadRequestWithMsg(ref err, ref msg) => {
                let response_body = ServiceErrorBody {
                    msg: Some(format!("BadRequest: {}, Msg: {}", err, msg))
                };
                HttpResponse::BadRequest().json(response_body)
            }
            ServiceError::Unauthorized(ref err) => {
                let response_body = ServiceErrorBody {
                    msg: Some(format!("Unauthorized: {}", err))
                };
                HttpResponse::Unauthorized().json(response_body)
            }
            ServiceError::UnauthorizedWithMsg(ref err, ref msg) => {
                let response_body = ServiceErrorBody {
                    msg: Some(format!("Unauthorized: {}, Msg: {}", err, msg))
                };
                HttpResponse::Unauthorized().json(response_body)
            }
        }
    }
}

impl From<JudgeCoreError> for ServiceError {
    fn from(value: JudgeCoreError) -> Self {
        return Self::InternalError(anyhow::anyhow!("{:?}", value))
    }
}
