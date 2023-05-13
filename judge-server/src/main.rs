mod service;

use actix_web::{App, HttpServer};
use utoipa::OpenApi;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    tokio::spawn(async move {
        // Suppose to send heartbeat here to a remote host
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });

    HttpServer::new(|| {
        App::new().configure(service::route).service(
            utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![(
                utoipa_swagger_ui::Url::new("api", "/api-docs/openapi.json"),
                service::ApiDoc::openapi(),
            )]),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
