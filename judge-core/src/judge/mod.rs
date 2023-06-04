use crate::{compiler::Language, run::sandbox::RlimitConfigs};

pub mod common;
pub mod interact;

#[derive(Debug, Clone)]
pub struct JudgeConfig {
    pub language: Language,
    pub program_path: String,
    pub custom_checker_path: Option<String>,
    pub input_file_path: String,
    pub output_file_path: String,
    pub answer_file_path: String,
    pub check_file_path: String,
    pub rlimit_configs: RlimitConfigs,
}
