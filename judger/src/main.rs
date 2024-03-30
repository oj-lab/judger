mod agent;
mod error;
mod handler;
mod option;
mod worker;

#[macro_use]
extern crate serde_derive;
extern crate lazy_static;

use actix_web::{web::Data, App, HttpServer};
use agent::platform;
use worker::JudgeWorker;

#[actix_web::main]
// The button provided by rust-analyzer will not work as expected here
// Use RUN AND DEBUG feature in VSCode
async fn main() -> std::io::Result<()> {
    let opt = option::load_option();

    // TODO: Send heartbeat here to a remote host

    let platform_client = platform::PlatformClient::new(opt.platform_uri.clone());
    let maybe_rclone_client = if opt.enable_rclone {
        Some(agent::rclone::RcloneClient::new(
            opt.rclone_config_path.clone(),
        ))
    } else {
        None
    };

    let worker = match JudgeWorker::new(
        platform_client,
        maybe_rclone_client,
        opt.fetch_task_interval,
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
