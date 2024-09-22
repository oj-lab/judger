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
            match platform_client.pick_judge_task().await {
                Ok(maybe_task) => {
                    if maybe_task.is_none() {
                        continue;
                    }
                    let task = maybe_task.unwrap();
                    log::info!("Received task: {:?}", task);

                    // TODO: handle failure for set_busy here & return the task to the queue
                    let _ = state::set_busy();

                    let prepare_result = self.prepare_judge(
                        task.problem_slug.clone(),
                        task.language,
                        task.code.clone(),
                    );
                    if let Err(e) = prepare_result {
                        log::debug!("Failed to prepare judge: {:?}", e);
                        let mut verdict = JudgeVerdict::SystemError;
                        if let JudgeCoreError::CompileError(_) = e {
                            verdict = JudgeVerdict::CompileError;
                        }
                        let _ = platform_client
                            .report_judge_task(
                                &task.judge_uid,
                                &task.redis_stream_id.clone(),
                                verdict,
                            )
                            .await
                            .map_err(|e| {
                                log::debug!("Failed to report judge task: {:?}", e);
                            });
                        continue;
                    }
                    let judge: JudgeBuilder = prepare_result.unwrap();
                    let _ = platform_client
                        .report_judge_result_count(&task.judge_uid, judge.testdata_configs.len())
                        .await
                        .map_err(|e| {
                            log::warn!("Failed to report judge result count: {:?}", e);
                        });

                    let mut verdict = JudgeVerdict::Accepted;
                    for idx in 0..judge.testdata_configs.len() {
                        log::debug!(
                            "Judge {}, {}, Testcase {}!",
                            task.redis_stream_id,
                            task.problem_slug,
                            idx
                        );
                        let judge_config = JudgeConfig {
                            test_data: judge.testdata_configs[idx].clone(),
                            program: judge.program_config.clone(),
                            checker: judge.checker_config.clone(),
                            runtime: judge.runtime_config.clone(),
                        };

                        let judge_result = self.run_judge(judge_config);
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

                        let _ = platform_client
                            .report_judge_result(
                                &task.judge_uid,
                                result.verdict.clone(),
                                result.time_usage.as_millis() as usize,
                                result.memory_usage_bytes as usize,
                            )
                            .await
                            .map_err(|e| {
                                log::warn!("Failed to report judge result count: {:?}", e);
                            });
                        if result.verdict != JudgeVerdict::Accepted {
                            verdict = result.verdict;
                            break;
                        }
                    }

                    let _ = platform_client
                        .report_judge_task(&task.judge_uid, &task.redis_stream_id.clone(), verdict)
                        .await
                        .map_err(|e| {
                            log::debug!("Failed to report judge task: {:?}", e);
                        });

                    state::set_idle()
                }
                Err(e) => log::debug!("Error sending request: {:?}", e),
            }
        }
    }

    pub fn prepare_judge(
        &self,
        problem_slug: String,
        language: Language,
        code: String,
    ) -> Result<JudgeBuilder, JudgeCoreError> {
        if let Some(rclone_client) = self.maybe_rclone_client.as_ref() {
            rclone_client.sync_bucket(&self.package_bucket, &self.package_dir)?;
        }

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

        let builder = JudgeBuilder::new(JudgeBuilderInput {
            package_type: PackageType::ICPC,
            package_path: problem_package_dir,
            runtime_path: runtime_path.clone(),
            src_language: language,
            src_path: runtime_path.clone().join(&src_file_name),
        })?;
        log::info!("Builder created success: {:?}", builder);
        Ok(builder)
    }

    pub fn run_judge(&self, judge_config: JudgeConfig) -> Result<JudgeResultInfo, anyhow::Error> {
        judge::common::run_judge(&judge_config)
            .map_err(|e| anyhow::anyhow!("Failed to run judge: {:?}", e))
    }
}
