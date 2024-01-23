use crate::service::error::JudgeServiceError;
use anyhow;
use std::path::Path;
use std::process::Command;

fn check_rclone(config_path: &Path) -> Result<(), JudgeServiceError> {
    let mut cmd = Command::new("rclone");
    let status = cmd
        .arg("--config")
        .arg(config_path)
        .arg("ls")
        .arg("minio:")
        .status()
        .expect("Failed to rclone");
    if status.success() {
        Ok(())
    } else {
        Err(JudgeServiceError::RcloneError(anyhow::anyhow!(
            "rclone failed, please check config."
        )))
    }
}

pub const PACKAGE_SAVE_DIRNAME: &str = "rclone-problem-package";
pub const RCLONE_CONFIG_FILE: &str = "rclone-minio.conf";

pub fn sync_package(data_dir: &Path, bucket_name: &str) -> Result<(), JudgeServiceError> {
    let config_path = data_dir.join(RCLONE_CONFIG_FILE);
    let package_path = data_dir.join(PACKAGE_SAVE_DIRNAME);
    let check_res = check_rclone(&config_path);
    if check_res.as_ref().is_err() {
        return check_res;
    }
    println!("{:?}", data_dir);
    let mut cmd = Command::new("rclone");
    let status = cmd
        .arg("--config")
        .arg(format!("{}", config_path.to_string_lossy()))
        .arg("sync")
        .arg(format!("minio:{}", bucket_name))
        .arg(format!("{}", package_path.to_string_lossy()))
        .status()
        .expect("Failed to rclone");
    if status.success() {
        Ok(())
    } else {
        Err(JudgeServiceError::RcloneError(anyhow::anyhow!(
            "rclone sync failed, please check config."
        )))
    }
}
