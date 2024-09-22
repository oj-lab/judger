use super::http::HttpClient;
use judge_core::{compiler::Language, judge::result::JudgeVerdict};

pub struct PlatformClient {
    client: HttpClient,
}

impl PlatformClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: HttpClient::new(base_url),
        }
    }

    pub async fn pick_judge_task(&self) -> Result<Option<JudgeTask>, anyhow::Error> {
        pick_judge_task(&self.client).await
    }

    pub async fn report_judge_result_count(
        &self,
        judge_uid: &str,
        result_count: usize,
    ) -> Result<(), anyhow::Error> {
        report_judge_result_count(&self.client, judge_uid, result_count).await
    }

    pub async fn report_judge_result(
        &self,
        judge_uid: &str,
        verdict: JudgeVerdict,
        time_usage_ms: usize,
        memory_usage_bytes: usize,
    ) -> Result<(), anyhow::Error> {
        report_judge_result(
            &self.client,
            judge_uid,
            verdict,
            time_usage_ms,
            memory_usage_bytes,
        )
        .await
    }

    pub async fn report_judge_task(
        &self,
        judge_uid: &str,
        stream_id: &str,
        verdict: JudgeVerdict,
    ) -> Result<(), anyhow::Error> {
        report_task(&self.client, judge_uid, stream_id, verdict).await
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct JudgeTask {
    #[serde(rename = "judgeUID")]
    pub judge_uid: String,
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

async fn pick_judge_task(client: &HttpClient) -> Result<Option<JudgeTask>, anyhow::Error> {
    let pick_url = "api/v1/judge/task/pick";
    let body = PickTaskBody {
        consumer: "".to_string(),
    };
    let response = client
        .post(pick_url.to_string())?
        .json(&body)
        .send()
        .await?;

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
struct ReportJudgeResultCountBody {
    #[serde(rename = "judgeUID")]
    judge_uid: String,
    #[serde(rename = "resultCount")]
    result_count: usize,
}

async fn report_judge_result_count(
    client: &HttpClient,
    judge_uid: &str,
    result_count: usize,
) -> Result<(), anyhow::Error> {
    let report_url = "api/v1/judge/task/report/result-count";
    let body = ReportJudgeResultCountBody {
        judge_uid: judge_uid.to_owned(),
        result_count,
    };
    let response = client
        .put(report_url.to_string())?
        .json(&body)
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::OK => Ok(()),
        _ => Err(anyhow::anyhow!("Report JudgeResultCount Failed")),
    }
}

#[derive(Serialize)]
struct ReportJudgeResultBody {
    #[serde(rename = "judgeUID")]
    judge_uid: String,
    verdict: JudgeVerdict,
    #[serde(rename = "timeUsageMS")]
    time_usage_ms: usize,
    #[serde(rename = "memoryUsageBytes")]
    memory_usage_bytes: usize,
}

async fn report_judge_result(
    client: &HttpClient,
    judge_uid: &str,
    verdict: JudgeVerdict,
    time_usage_ms: usize,
    memory_usage_bytes: usize,
) -> Result<(), anyhow::Error> {
    let report_url = "api/v1/judge/task/report/result";
    let body = ReportJudgeResultBody {
        judge_uid: judge_uid.to_owned(),
        verdict,
        time_usage_ms,
        memory_usage_bytes,
    };
    let response = client
        .post(report_url.to_string())?
        .json(&body)
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::OK => Ok(()),
        _ => Err(anyhow::anyhow!("Report JudgeResult Failed")),
    }
}

#[derive(Serialize)]
struct ReportJudgeTaskBody {
    #[serde(rename = "judgeUID")]
    judge_uid: String,
    consumer: String,
    #[serde(rename = "redisStreamID")]
    redis_stream_id: String,
    verdict: JudgeVerdict,
}
#[derive(Deserialize, Debug)]
struct ReportJudgeTaskResponse {
    message: String,
}

async fn report_task(
    client: &HttpClient,
    judge_uid: &str,
    stream_id: &str,
    verdict: JudgeVerdict,
) -> Result<(), anyhow::Error> {
    let report_url = "api/v1/judge/task/report";
    let body = ReportJudgeTaskBody {
        judge_uid: judge_uid.to_owned(),
        consumer: "".to_string(),
        redis_stream_id: stream_id.to_owned(),
        verdict,
    };
    let response = client
        .put(report_url.to_string())?
        .json(&body)
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::OK => {
            log::debug!(
                "Report message: {:?}",
                response.json::<ReportJudgeTaskResponse>().await?.message
            );
            Ok(())
        }
        _ => Err(anyhow::anyhow!("Report Failed")),
    }
}
