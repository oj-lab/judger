use crate::error::ServiceError;
use actix_web::{post, web, HttpResponse};
use utoipa::ToSchema;

#[derive(utoipa::OpenApi)]
#[openapi(paths(run_judge), components(schemas(RunJudgeBody)))]
pub struct JudgeApiDoc;

pub fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/judge").service(run_judge));
}

// TODO: Remove the first `_` when the segment is actually used
#[derive(Debug, ToSchema, Deserialize)]
pub struct RunJudgeBody {
    _src: String,
    _package_slug: String,
}

#[utoipa::path(
    context_path = "/api/v1/judge",
    request_body(content = RunJudgeBody, content_type = "application/json", description = "The info a judge task should refer to"),
    responses(
        (status = 200, description = "Judge run successfully")
    )
)]
#[post("")]
pub async fn run_judge(body: web::Json<RunJudgeBody>) -> Result<HttpResponse, ServiceError> {
    log::debug!("receive body: {:?}", body);

    Ok(HttpResponse::Ok().finish())
}
