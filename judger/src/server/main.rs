mod environment;
mod error;
mod service;

#[macro_use]
extern crate serde_derive;
extern crate lazy_static;

use actix_web::{web::Data, App, HttpServer};
use utoipa::OpenApi;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let opt = environment::load_option();
    environment::setup_logger();

    // Suppose to send heartbeat here to a remote host
    // tokio::spawn(async move {
    //     loop {
    //         tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    //         log::debug!("JudgeSever heartbeat")
    //     }
    // });

    let port = opt.port;

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(Data::new(opt.problem_package_dir.clone()))
            .configure(service::route)
            .service(
                utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![(
                    utoipa_swagger_ui::Url::new("api", "/api-docs/openapi.json"),
                    service::ApiDoc::openapi(),
                )]),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
