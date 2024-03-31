use anyhow::{self, Error};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct RcloneClient {
    config_path: PathBuf,
}

impl RcloneClient {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    pub fn is_avaliable(&self) -> bool {
        let mut binding = Command::new("rclone");
        let command = binding
            .arg("--config")
            .arg(format!("{}", self.config_path.to_string_lossy()))
            .arg("--contimeout")
            .arg("5s")
            // HTTP request is a typical low level operation, which will retry for many times
            // https://rclone.org/docs/#low-level-retries-number
            // Here we set it to 3 to avoid long waiting time
            .arg("--low-level-retries")
            .arg("3")
            .arg("ls")
            .arg("minio:");
        log::debug!("Checking rclone with command: {:?}", command);
        let status = command.status().expect("Failed to rclone");

        status.success()
    }

    pub fn sync_bucket(&self, bucket_name: &str, target_dir: &Path) -> Result<(), Error> {
        let mut binding = Command::new("rclone");
        let command = binding
            .arg("--config")
            .arg(format!("{}", self.config_path.to_string_lossy()))
            .arg("sync")
            .arg(format!("minio:{}", bucket_name))
            .arg(format!("{}", target_dir.to_string_lossy()));
        log::debug!("Syncing bucket with command: {:?}", command);
        let status = command.status().expect("Failed to rclone");
        if status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("rclone sync failed, please check config."))
        }
    }
}
