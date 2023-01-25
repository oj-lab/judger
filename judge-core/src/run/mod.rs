use std::os::unix::prelude::RawFd;

pub mod runner;
pub mod judger;

#[derive(Default)]
pub struct ResourceLimitConfig {
    pub stack_limit: Option<(u64, u64)>,
    pub as_limit: Option<(u64, u64)>,
    pub cpu_limit: Option<(u64, u64)>,
    pub nproc_limit: Option<(u64, u64)>,
    pub fsize_limit: Option<(u64, u64)>,
}

pub struct RunConfig {
    pub program_path: Option<String>,
    pub input_fd: RawFd,
    pub output_fd: RawFd,
    pub rlimit_config: Option<ResourceLimitConfig>,
}

