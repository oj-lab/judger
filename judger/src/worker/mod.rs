use crate::agent::platform::PlatformClient;
use crate::agent::rclone::RcloneClient;
use crate::handler::state;
use anyhow::Error;
use judge_core::compiler::Language;
use judge_core::error::JudgeCoreError;
use judge_core::judge;
use judge_core::judge::result::JudgeVerdict;
use judge_core::{
    judge::builder::{JudgeBuilder, JudgeBuilderInput},
    judge::result::JudgeResultInfo,
    judge::JudgeConfig,
    package::PackageType,
};
use std::time::Duration;
use std::{fs, path::PathBuf};
use tokio::time::interval;

pub struct JudgeWorker {
    maybe_platform_client: Option<PlatformClient>,
    interval_sec: u64,
    maybe_rclone_client: Option<RcloneClient>,
    package_bucket: String,
    package_dir: PathBuf,
}

impl JudgeWorker {
    pub fn new(
        maybe_platform_client: Option<PlatformClient>,
        maybe_rclone_client: Option<RcloneClient>,
        interval_sec: u64,
        package_bucket: String,
        package_dir: PathBuf,
    ) -> Result<Self, Error> {
        if let Some(rclone_client) = maybe_rclone_client.as_ref() {
            if rclone_client.is_avaliable() {
                rclone_client.sync_bucket(&package_bucket, &package_dir)?;
            } else {
                log::error!("Rclone client is not available");
                return Err(anyhow::anyhow!("Rclone client is not available"));
            }
        }
        Ok(Self {
            maybe_platform_client,
            maybe_rclone_client,
            interval_sec,
            package_bucket,
            package_dir,
        })
    }

    pub async fn run(&self) {
        log::info!("judge task worker started");

        if self.maybe_platform_client.is_none() {
            log::error!("Platform client is not available");
            return;
        }
        let platform_client = self.maybe_platform_client.as_ref().unwrap();

        let mut interval = interval(Duration::from_secs(self.interval_sec));
        loop {
            interval.tick().await;
            match platform_client.pick_task().await {
                Ok(task) => {
                    log::info!("Received task: {:?}", task);
                    match self.run_judge(
                        task.problem_slug.clone(),
                        task.language,
                        task.code.clone(),
                    ) {
                        Ok(results) => {
                            let report_response = platform_client
                                .report_task(&task.redis_stream_id.clone(), results)
                                .await;
                            if report_response.is_err() {
                                log::debug!(
                                    "Report failed with error: {:?}",
                                    report_response.err()
                                );
                                return;
                            }
                            log::info!(
                                "Submission {:?} report success",
                                task.submission_uid.clone()
                            );
                        }
                        Err(e) => log::info!("Error judge task: {:?}", e),
                    }
                }
                Err(e) => log::debug!("Error sending request: {:?}", e),
            }
        }
    }

    pub fn run_judge(
        &self,
        problem_slug: String,
        language: Language,
        code: String,
    ) -> Result<Vec<JudgeResultInfo>, anyhow::Error> {
        if let Some(rclone_client) = self.maybe_rclone_client.as_ref() {
            rclone_client.sync_bucket(&self.package_bucket, &self.package_dir)?;
        }

        state::set_busy()?;
        let problem_package_dir = self.package_dir.join(problem_slug);

        let uuid = uuid::Uuid::new_v4();
        let runtime_path = PathBuf::from("/tmp").join(uuid.to_string());
        let src_file_name = format!("src.{}", language.get_extension());
        log::debug!("runtime_path: {:?}", runtime_path);
        fs::create_dir_all(runtime_path.clone()).map_err(|e| {
            log::debug!("Failed to create runtime dir: {:?}", e);
            anyhow::anyhow!("Failed to create runtime dir")
        })?;
        fs::write(runtime_path.clone().join(&src_file_name), code.clone()).map_err(|e| {
            log::debug!("Failed to write src file: {:?}", e);
            anyhow::anyhow!("Failed to write src file")
        })?;

        let new_builder_result = JudgeBuilder::new(JudgeBuilderInput {
            package_type: PackageType::ICPC,
            package_path: problem_package_dir,
            runtime_path: runtime_path.clone(),
            src_language: language,
            src_path: runtime_path.clone().join(&src_file_name),
        });
        if let Err(e) = new_builder_result {
            if let JudgeCoreError::CompileError(_) = e {
                return Ok(vec![
                    JudgeResultInfo {
                        verdict: JudgeVerdict::CompileError,
                        time_usage: Duration::new(0, 0),
                        memory_usage_bytes: -1,
                        exit_status: -1,
                        checker_exit_status: -1,
                    };
                    1
                ]);
            }
            return Err(anyhow::anyhow!("Failed to create builder: {:?}", e));
        }
        let builder = new_builder_result.expect("builder creater error");
        log::debug!("Builder created: {:?}", builder);
        let mut results: Vec<JudgeResultInfo> = vec![];
        for idx in 0..builder.testdata_configs.len() {
            let judge_config = JudgeConfig {
                test_data: builder.testdata_configs[idx].clone(),
                program: builder.program_config.clone(),
                checker: builder.checker_config.clone(),
                runtime: builder.runtime_config.clone(),
            };
            let result = judge::common::run_judge(&judge_config).map_err(|e| {
                state::set_idle();
                anyhow::anyhow!("Failed to run judge: {:?}", e)
            })?;
            log::debug!("Judge result: {:?}", result);
            results.push(result);
        }

        log::debug!("Judge finished");
        state::set_idle();
        Ok(results)
    }
}
