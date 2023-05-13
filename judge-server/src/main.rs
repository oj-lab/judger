use actix_web::{App, HttpServer};
use utoipa::OpenApi;

mod api {
    use actix_web::{get, web, Responder};

    #[utoipa::path(
        responses(
            (status = 200, description = "Hello {name}!", body = String)
        )
    )]
    #[get("/hello/{name}")]
    async fn greet(name: web::Path<String>) -> impl Responder {
        format!("Hello {name}!")
    }
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    tokio::spawn(async move {
        // Suppose to send heartbeat here to a remote host
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            println!("Hello from Tokio!");
        }
    });

    #[derive(utoipa::OpenApi)]
    #[openapi(paths(api::greet))]
    struct ApiDoc;

    HttpServer::new(|| App::new()
            .service(api::greet)
            .service(utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![
                (
                    utoipa_swagger_ui::Url::new("api", "/api-docs/openapi.json"),
                    ApiDoc::openapi(),
                ),
            ]))
        )
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
