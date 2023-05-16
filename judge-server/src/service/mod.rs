mod greet;
mod judge;

use actix_web::web;
use utoipa::OpenApi;

#[derive(utoipa::OpenApi)]
#[openapi(external_docs(
    url = "/swagger-ui/?urls.primaryName=judge",
    description = "Start judge"
))]
pub struct ApiDoc;

pub fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(judge::route)
            .service(greet::greet),
    )
    .service(
        utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![
            (
                utoipa_swagger_ui::Url::new("root", "/api-docs/openapi.json"),
                ApiDoc::openapi(),
            ),
            (
                utoipa_swagger_ui::Url::new("judge", "/api-docs/judge.json"),
                judge::JudgeApiDoc::openapi(),
            ),
        ]),
    );
}
