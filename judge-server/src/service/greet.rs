use actix_web::{get, web, Responder};

#[utoipa::path(
    context_path = "/api",
    responses(
        (status = 200, description = "Hello {name}!", body = String)
    )
)]
#[get("/hello/{name}")]
pub async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}
