mod agent;
mod env;
mod error;
mod handler;
mod worker;

#[macro_use]
extern crate serde_derive;
extern crate lazy_static;

use actix_web::{web::Data, App, HttpServer};
use worker::JudgeWorker;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let opt = env::load_option();
    env::setup_logger();

    // TODO: Send heartbeat here to a remote host

    let worker = match JudgeWorker::new(
        opt.platform_uri,
        opt.fetch_task_interval,
        opt.rclone_config,
        opt.problem_package_bucket.clone(),
        opt.problem_package_dir.clone(),
    ) {
        Ok(maybe_worker) => {
            if let Some(worker) = maybe_worker {
                worker
            } else {
                log::error!("Failed to create worker");
                return Ok(());
            }
        }
        Err(e) => {
            log::error!("Failed to create worker: {:?}", e);
            return Ok(());
        }
    };
    tokio::spawn(async move { worker.run().await });

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(Data::new(opt.problem_package_dir.clone()))
            .configure(handler::route)
    })
    .bind(("0.0.0.0", opt.port))?
    .run()
    .await
}
