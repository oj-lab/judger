use anyhow::anyhow;
use std::{fs, io, path::PathBuf};

pub fn copy_recursively(src: &PathBuf, dest: &PathBuf) -> io::Result<()> {
    log::debug!("copying {:?} to {:?}", src, dest);
    if fs::metadata(src)?.is_file() {
        fs::copy(src, dest)?;
    } else {
        if !dest.exists() || !fs::metadata(dest)?.is_dir() {
            log::debug!("creating dir: {:?}", dest);
            fs::create_dir_all(dest)?;
        }
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let file_name = src_path.file_name().unwrap();
            let dest_path = dest.join(file_name);
            copy_recursively(&src_path, &dest_path)?;
        }
    }

    Ok(())
}

use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::error::JudgeCoreError;

pub fn compare_files(file_path1: &PathBuf, file_path2: &PathBuf) -> bool {
    let file1 = BufReader::new(File::open(file_path1).unwrap());
    let file2 = BufReader::new(File::open(file_path2).unwrap());

    file1.lines().zip(file2.lines()).all(|(line1, line2)| {
        // Ignore any trailing whitespace or newline characters
        let line1_string = line1.unwrap();
        let line2_string: String = line2.unwrap();
        let trimed1 = line1_string.trim_end();
        let trimed2 = line2_string.trim_end();
        trimed1 == trimed2
    })
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
