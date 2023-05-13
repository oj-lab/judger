pub mod greet;

use actix_web::web;
use utoipa;

#[derive(utoipa::OpenApi)]
#[openapi(paths(greet::greet))]
pub struct ApiDoc;

pub fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api").service(greet::greet));
}
