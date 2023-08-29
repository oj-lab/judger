use anyhow::anyhow;
use std::path::PathBuf;

use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::error::JudgeCoreError;

pub fn compare_files(file_path1: &PathBuf, file_path2: &PathBuf) -> bool {
    log::debug!("Comparing output files");
    let file1 = BufReader::new(File::open(file_path1).unwrap());
    let file2 = BufReader::new(File::open(file_path2).unwrap());

    let file1_content: String = file1.lines().map(|l| l.unwrap()).collect();
    let file2_content: String = file2.lines().map(|l| l.unwrap()).collect();

    file1_content.trim_end() == file2_content.trim_end()
}

pub fn get_pathbuf_str(path: &PathBuf) -> Result<String, JudgeCoreError> {
    match path.to_str() {
        Some(path_str) => Ok(path_str.to_owned()),
        None => Err(JudgeCoreError::AnyhowError(anyhow!(
            "PathBuf to str failed: {:?}",
            path
        ))),
    }
}
