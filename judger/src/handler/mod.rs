mod greet;
pub mod state;

use actix_web::web;
use utoipa::OpenApi;

#[derive(utoipa::OpenApi)]
#[openapi(external_docs(
    url = "/swagger-ui/?urls.primaryName=judge",
    description = "Judger API docs",
))]
pub struct ApiDoc;

pub fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(greet::greet)
            .configure(state::route),
    )
    .service(
        utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![
            (
                utoipa_swagger_ui::Url::new("root", "/api-docs/openapi.json"),
                ApiDoc::openapi(),
            ),
            (
                utoipa_swagger_ui::Url::new("state", "/api-docs/state.json"),
                state::StateApiDoc::openapi(),
            ),
        ]),
    );
}
