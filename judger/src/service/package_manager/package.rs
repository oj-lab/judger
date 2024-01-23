use crate::service::error::JudgeServiceError;
use anyhow;
use std::process::Command;

fn check_rclone() -> Result<(), JudgeServiceError> {
    let mut cmd = Command::new("rclone");
    let status = cmd
        .arg("--config")
        .arg("../data/rclone-minio.conf")
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

pub fn sync_package() -> Result<(), JudgeServiceError> {
    let check_res = check_rclone();
    if check_res.as_ref().is_err() {
        return check_res;
    }
    let mut cmd = Command::new("rclone");
    let bucket_name = "oj-lab-problem-package";
    let status = cmd
        .arg("--config")
        .arg("../data/rclone-minio.conf")
        .arg("sync")
        .arg(format!("minio:{}", bucket_name))
        .arg("../data/rclone-problem-package")
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
