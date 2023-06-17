use std::path::PathBuf;

use crate::run::{executor::Executor, sandbox::RlimitConfigs};

pub mod common;
pub mod interact;
pub mod result;

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub rlimit_configs: RlimitConfigs,
}

/// When `executor` is `None`, default checker will be used.
#[derive(Debug, Clone)]
pub struct CheckerConfig {
    pub executor: Option<Executor>,
    pub output_file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ProgramConfig {
    pub executor: Executor,
    pub output_file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TestDataConfig {
    pub input_file_path: PathBuf,
    pub answer_file_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct JudgeConfig {
    pub test_data: TestDataConfig,
    pub runtime: RuntimeConfig,
    pub program: ProgramConfig,
    pub checker: CheckerConfig,
}
