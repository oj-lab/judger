mod httpclient;
use crate::error::ClientError;
use crate::service::state;
use httpclient::HttpClient;
use judge_core::judge;
use judge_core::{
    compiler::Language,
    judge::builder::{JudgeBuilder, JudgeBuilderInput},
    judge::result::JudgeResultInfo,
    judge::JudgeConfig,
    package::PackageType,
};
use judger::service::package_manager::package;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::{fs, path::PathBuf};
use tokio::time::interval;

#[derive(Serialize)]
struct PickBody {
    consumer: String,
}
#[derive(Deserialize, Debug)]
struct PickResponse {
    task: JudgeTask,
}
#[derive(Serialize)]
struct ReportBody {
    consumer: String,
    stream_id: String,
    verdict_json: String,
}
#[derive(Deserialize, Debug)]
struct ReportResponse {
    message: String,
}
#[derive(Deserialize, Debug)]
struct JudgeTask {
    #[serde(rename = "submissionUID")]
    submission_uid: String,
    #[serde(rename = "problemSlug")]
    problem_slug: String,
    code: String,
    language: Language,
    #[serde(rename = "redisStreamID")]
    redis_stream_id: String,
}

pub async fn run_client(base_url: String, interval_sec: u64) {
    let mut interval = interval(Duration::from_secs(interval_sec));
    let client = HttpClient::new(base_url);

    loop {
        interval.tick().await;
        match pick_task(&client).await {
            Ok(task) => {
                let stream_id = task.redis_stream_id.clone();
                let submission_uid = task.submission_uid.clone();
                log::info!("Received task: {:?}", task);
                match run_judge(task) {
                    Ok(results) => {
                        let report_response = report_task(&client, &stream_id, results).await;
                        if report_response.is_err() {
                            log::debug!("Report failed {:?}", report_response);
                            return;
                        }
                        log::info!("Submission {:?} report success", submission_uid);
                    }
                    Err(e) => log::info!("Error judge task: {:?}", e),
                }
            }
            Err(e) => log::debug!("Error sending request: {:?}", e),
        }
    }
}

async fn pick_task(client: &HttpClient) -> Result<JudgeTask, ClientError> {
    let pick_url = "/task/pick";
    let body = PickBody {
        consumer: "".to_string(),
    };
    let response = client.post(pick_url.to_string()).json(&body).send().await?;

    match response.status() {
        reqwest::StatusCode::OK => Ok(response.json::<PickResponse>().await?.task),
        _ => Err(ClientError::InternalError(anyhow::anyhow!(
            "Queue is empty"
        ))),
    }
}

async fn report_task(
    client: &HttpClient,
    stream_id: &str,
    results: Vec<JudgeResultInfo>,
) -> Result<(), ClientError> {
    let report_url = "/task/report";
    let body = ReportBody {
        consumer: "".to_string(),
        stream_id: stream_id.to_owned(),
        verdict_json: serde_json::to_string(&results).unwrap(),
    };
    let response = client
        .post(report_url.to_string())
        .json(&body)
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::OK => {
            log::debug!(
                "Report message: {:?}",
                response.json::<ReportResponse>().await?.message
            );
            Ok(())
        }
        _ => Err(ClientError::InternalError(anyhow::anyhow!("Report Failed"))),
    }
}

fn run_judge(task: JudgeTask) -> Result<Vec<JudgeResultInfo>, ClientError> {
    if let Err(sync_err) = package::sync_package(&PathBuf::from("data"), "oj-lab-problem-package") {
        return Err(ClientError::PackageError(sync_err));
    };
    if let Err(set_err) = state::set_busy() {
        return Err(ClientError::InternalError(set_err));
    }
    let problem_package_dir = PathBuf::from(format!("data/{}", package::PACKAGE_SAVE_DIRNAME));
    let problem_slug = task.problem_slug;
    let uuid = uuid::Uuid::new_v4();
    let runtime_path = PathBuf::from("/tmp").join(uuid.to_string());
    let src_file_name = format!("src.{}", task.language.get_extension());
    log::debug!("runtime_path: {:?}", runtime_path);
    fs::create_dir_all(runtime_path.clone()).map_err(|e| {
        log::debug!("Failed to create runtime dir: {:?}", e);
        ClientError::InternalError(anyhow::anyhow!("Failed to create runtime dir"))
    })?;
    fs::write(runtime_path.clone().join(&src_file_name), task.code.clone()).map_err(|e| {
        log::debug!("Failed to write src file: {:?}", e);
        ClientError::InternalError(anyhow::anyhow!("Failed to write src file"))
    })?;

    let new_builder_result = JudgeBuilder::new(JudgeBuilderInput {
        package_type: PackageType::ICPC,
        package_path: problem_package_dir.join(problem_slug.clone()),
        runtime_path: runtime_path.clone(),
        src_language: task.language,
        src_path: runtime_path.clone().join(&src_file_name),
    });
    if new_builder_result.is_err() {
        state::set_idle();
        return Err(ClientError::InternalError(anyhow::anyhow!(
            "Failed to new builder result: {:?}",
            new_builder_result.err()
        )));
    }
    let builder = new_builder_result?;
    log::debug!("Builder created: {:?}", builder);
    let mut results: Vec<JudgeResultInfo> = vec![];
    for idx in 0..builder.testdata_configs.len() {
        let judge_config = JudgeConfig {
            test_data: builder.testdata_configs[idx].clone(),
            program: builder.program_config.clone(),
            checker: builder.checker_config.clone(),
            runtime: builder.runtime_config.clone(),
        };
        let result = judge::common::run_judge(&judge_config)?;
        log::debug!("Judge result: {:?}", result);
        results.push(result);
    }

    log::debug!("Judge finished");
    state::set_idle();
    Ok(results)
}
