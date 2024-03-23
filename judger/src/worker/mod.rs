use crate::agent::platform::{JudgeTask, PlatformClient};
use crate::agent::rclone::RcloneClient;
use crate::handler::state;
use anyhow::Error;
use judge_core::judge;
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
    platform_client: PlatformClient,
    interval_sec: u64,
    rclone_client: RcloneClient,
    package_bucket: String,
    package_dir: PathBuf,
}

impl JudgeWorker {
    pub fn new(
        platform_uri: String,
        interval_sec: u64,
        rclone_config: PathBuf,
        package_bucket: String,
        package_dir: PathBuf,
    ) -> Result<Option<Self>, Error> {
        let platform_client = PlatformClient::new(platform_uri);
        let rclone_client = RcloneClient::new(rclone_config);
        if !rclone_client.is_avaliable() {
            Err(anyhow::anyhow!("Rclone is not avaliable"))?;
        }
        Ok(Some(Self {
            platform_client,
            rclone_client,
            interval_sec,
            package_bucket,
            package_dir,
        }))
    }

    pub async fn run(&self) {
        let _ = self
            .rclone_client
            .sync_bucket(&self.package_bucket, &self.package_dir)
            .map_err(|e| log::debug!("Failed to sync bucket: {:?}", e));

        let mut interval = interval(Duration::from_secs(self.interval_sec));
        loop {
            interval.tick().await;
            match self.platform_client.pick_task().await {
                Ok(task) => {
                    log::info!("Received task: {:?}", task);
                    match self.run_judge(&task) {
                        Ok(results) => {
                            let report_response = self
                                .platform_client
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

    fn run_judge(&self, task: &JudgeTask) -> Result<Vec<JudgeResultInfo>, anyhow::Error> {
        self.rclone_client
            .sync_bucket(&self.package_bucket, &self.package_dir)?;
        state::set_busy()?;
        let problem_package_dir = self.package_dir.join(&task.problem_slug);

        let uuid = uuid::Uuid::new_v4();
        let runtime_path = PathBuf::from("/tmp").join(uuid.to_string());
        let src_file_name = format!("src.{}", task.language.get_extension());
        log::debug!("runtime_path: {:?}", runtime_path);
        fs::create_dir_all(runtime_path.clone()).map_err(|e| {
            log::debug!("Failed to create runtime dir: {:?}", e);
            anyhow::anyhow!("Failed to create runtime dir")
        })?;
        fs::write(runtime_path.clone().join(&src_file_name), task.code.clone()).map_err(|e| {
            log::debug!("Failed to write src file: {:?}", e);
            anyhow::anyhow!("Failed to write src file")
        })?;

        let new_builder_result = JudgeBuilder::new(JudgeBuilderInput {
            package_type: PackageType::ICPC,
            package_path: problem_package_dir,
            runtime_path: runtime_path.clone(),
            src_language: task.language,
            src_path: runtime_path.clone().join(&src_file_name),
        })
        .map_err(|e| {
            state::set_idle();
            anyhow::anyhow!("Failed to new builder result: {:?}", e)
        });
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
