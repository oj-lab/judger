use crate::{compiler::Language, sandbox::ResourceLimitConfig};

pub mod common;
pub mod interact;

pub struct JudgeConfig {
    pub language: Language,
    pub program_path: String,
    pub custom_checker_path: Option<String>,
    pub input_file_path: String,
    pub output_file_path: String,
    pub answer_file_path: String,
    pub check_file_path: String,
    pub rlimit_config: ResourceLimitConfig,
}
