use super::http::HttpClient;
use judge_core::{compiler::Language, judge::result::JudgeResultInfo};

pub struct PlatformClient {
    client: HttpClient,
}

impl PlatformClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: HttpClient::new(base_url),
        }
    }

    pub async fn pick_task(&self) -> Result<Option<JudgeTask>, anyhow::Error> {
        pick_task(&self.client).await
    }

    pub async fn report_task(
        &self,
        stream_id: &str,
        results: Vec<JudgeResultInfo>,
    ) -> Result<(), anyhow::Error> {
        report_task(&self.client, stream_id, results).await
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct JudgeTask {
    #[serde(rename = "submissionUID")]
    pub submission_uid: String,
    #[serde(rename = "problemSlug")]
    pub problem_slug: String,
    pub code: String,
    pub language: Language,
    #[serde(rename = "redisStreamID")]
    pub redis_stream_id: String,
}
#[derive(Serialize)]
struct PickTaskBody {
    consumer: String,
}
#[derive(Deserialize, Debug)]
struct PickTaskResponse {
    task: JudgeTask,
}

async fn pick_task(client: &HttpClient) -> Result<Option<JudgeTask>, anyhow::Error> {
    let pick_url = "api/v1/judge/task/pick";
    let body = PickTaskBody {
        consumer: "".to_string(),
    };
    let response = client.post(pick_url.to_string()).json(&body).send().await?;

    match response.status() {
        reqwest::StatusCode::OK => Ok(Some(response.json::<PickTaskResponse>().await?.task)),
        reqwest::StatusCode::NO_CONTENT => Ok(None),
        _ => {
            log::error!("Failed to pick task: {:?}", response);
            Err(anyhow::anyhow!(format!(
                "Failed to pick task: {:?}",
                response
            )))
        }
    }
}

#[derive(Serialize)]
struct ReportTaskBody {
    consumer: String,
    stream_id: String,
    verdict_json: String,
}
#[derive(Deserialize, Debug)]
struct ReportTaskResponse {
    message: String,
}

async fn report_task(
    client: &HttpClient,
    stream_id: &str,
    results: Vec<JudgeResultInfo>,
) -> Result<(), anyhow::Error> {
    let report_url = "api/v1/judge/task/report";
    let body = ReportTaskBody {
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
                response.json::<ReportTaskResponse>().await?.message
            );
            Ok(())
        }
        _ => Err(anyhow::anyhow!("Report Failed")),
    }
}
