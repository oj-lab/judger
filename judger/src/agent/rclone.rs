use anyhow::{self, Error};
use std::path::PathBuf;
use std::process::Command;

pub struct RcloneClient {
    config_path: PathBuf,
}

impl RcloneClient {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    pub fn is_avaliable(&self) -> bool {
        let status = Command::new("rclone")
            .arg("--config")
            .arg(format!("{}", self.config_path.to_string_lossy()))
            .arg("ls")
            .arg("minio:")
            .status()
            .expect("Failed to rclone");

        status.success()
    }

    pub fn sync_bucket(&self, bucket_name: &str, target_dir: PathBuf) -> Result<(), Error> {
        let status = Command::new("rclone")
            .arg("--config")
            .arg(format!("{}", self.config_path.to_string_lossy()))
            .arg("sync")
            .arg(format!("minio:{}", bucket_name))
            .arg(format!("{}", target_dir.to_string_lossy()))
            .status()
            .expect("Failed to rclone");
        if status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("rclone sync failed, please check config."))
        }
    }
}
