mod environment;
mod service;

use actix_web::{App, HttpServer};
use utoipa::OpenApi;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let opt = environment::load_option();
    environment::setup_logger();
    log::info!("{:?}", opt);

    // Suppose to send heartbeat here to a remote host
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            log::debug!("JudgeSever heartbeat")
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
    .bind(("127.0.0.1", opt.port))?
    .run()
    .await
}
