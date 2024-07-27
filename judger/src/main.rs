mod agent;
mod error;
mod handler;
mod option;
mod worker;

#[macro_use]
extern crate serde_derive;
extern crate lazy_static;

use std::{fs, path::PathBuf, time::Duration};

use actix_web::{App, HttpServer};
use agent::{platform, rclone::RcloneClient};
use judge_core::judge::{
    result::{JudgeResultInfo, JudgeVerdict},
    JudgeConfig,
};
use option::JudgerCommad;
use worker::JudgeWorker;

#[actix_web::main]
// The button provided by rust-analyzer will not work as expected here
// Use RUN AND DEBUG feature in VSCode
async fn main() -> std::io::Result<()> {
    let opt = option::load_option();

    let maybe_rclone_client = if opt.enable_rclone {
        Some(agent::rclone::RcloneClient::new(
            opt.rclone_config_path.clone(),
        ))
    } else {
        None
    };

    match opt.cmd {
        option::JudgerCommad::Serve {
            platform_uri,
            fetch_task_interval,
            port,
        } => {
            serve(
                maybe_rclone_client,
                opt.problem_package_bucket,
                opt.problem_package_dir,
                platform_uri.clone(),
                fetch_task_interval,
                port,
            )
            .await
        }
        JudgerCommad::Judge {
            problem_slug,
            language,
            src_path,
        } => {
            judge(
                maybe_rclone_client,
                opt.problem_package_bucket,
                opt.problem_package_dir,
                problem_slug,
                language,
                src_path,
            )
            .await
        }
    }
}

async fn serve(
    maybe_rclone_client: Option<RcloneClient>,
    problem_package_bucket: String,
    problem_package_dir: PathBuf,
    platform_uri: String,
    fetch_task_interval: u64,
    port: u16,
) -> std::io::Result<()> {
    let platform_client = platform::PlatformClient::new(platform_uri.clone());

    let worker = match JudgeWorker::new(
        Some(platform_client),
        maybe_rclone_client,
        fetch_task_interval,
        problem_package_bucket.clone(),
        problem_package_dir.clone(),
    ) {
        Ok(worker) => worker,
        Err(e) => {
            log::error!("Failed to create worker: {:?}", e);
            return Ok(());
        }
    };
    tokio::spawn(async move { worker.run().await });

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .configure(handler::route)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

async fn judge(
    maybe_rclone_client: Option<RcloneClient>,
    problem_package_bucket: String,
    problem_package_dir: PathBuf,
    problem_slug: String,
    language: judge_core::compiler::Language,
    src_path: std::path::PathBuf,
) -> std::io::Result<()> {
    // Read code from src_path
    let code = match fs::read_to_string(src_path) {
        Ok(code) => code,
        Err(e) => {
            log::error!("Failed to read code from src_path: {:?}", e);
            return Ok(());
        }
    };

    let worker = match JudgeWorker::new(
        None,
        maybe_rclone_client,
        0,
        problem_package_bucket.clone(),
        problem_package_dir.clone(),
    ) {
        Ok(worker) => worker,
        Err(e) => {
            log::error!("Failed to create worker: {:?}", e);
            return Ok(());
        }
    };

    let prepare_result = worker.prepare_judge(problem_slug.clone(), language, code.clone());
    if prepare_result.is_err() {
        log::error!("Failed to prepare judge: {:?}", prepare_result.err());
        return Ok(());
    }
    let judge = prepare_result.unwrap();

    let mut verdict = JudgeVerdict::Accepted;
    for idx in 0..judge.testdata_configs.len() {
        log::debug!("Judge {}, Testcase {}!", problem_slug, idx);
        let judge_config = JudgeConfig {
            test_data: judge.testdata_configs[idx].clone(),
            program: judge.program_config.clone(),
            checker: judge.checker_config.clone(),
            runtime: judge.runtime_config.clone(),
        };

        let judge_result = worker.run_judge(judge_config);
        let mut result = JudgeResultInfo {
            verdict: JudgeVerdict::SystemError,
            time_usage: Duration::from_secs(0),
            memory_usage_bytes: 0,
            exit_status: -1,
            checker_exit_status: -1,
        };
        match judge_result {
            Ok(r) => {
                result = r;
            }
            Err(e) => {
                log::debug!("Failed to run judge: {:?}", e);
            }
        }
        if result.verdict != JudgeVerdict::Accepted {
            verdict = result.verdict;
            break;
        }
    }
    println!("{:?}", verdict);
    Ok(())
}
