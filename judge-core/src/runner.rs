use crate::{
    error::JudgeCoreError,
    rules::{cpp_loader::CppLoader, get_default_kill_context, load_rules},
};
use nix::{
    errno::Errno,
    sys::resource::{
        setrlimit,
        Resource::{RLIMIT_AS, RLIMIT_CPU, RLIMIT_FSIZE, RLIMIT_NPROC, RLIMIT_STACK},
    },
    unistd::dup2,
    unistd::execve,
};
use std::ffi::CString;
use std::fs::File;
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};

pub struct RunnerConfig {
    pub program_path: String,
    pub input_file_path: String,
    pub output_file_path: String,
    pub rlimit_config: ResourceLimitConfig,
}

pub fn run_process(config: &RunnerConfig) -> Result<(), JudgeCoreError> {
    // TODO: Handle error
    set_resource_limit(&config.rlimit_config)?;

    let input_file = File::open(&config.input_file_path)?;
    let output_file = File::options()
        .write(true)
        .truncate(true) // Overwrite the whole content of this file
        .open(&config.output_file_path)
        .unwrap();

    let input_raw_fd: RawFd = input_file.as_raw_fd();
    let stdin_raw_fd: RawFd = io::stdin().as_raw_fd();
    dup2(input_raw_fd, stdin_raw_fd)?;
    let output_raw_fd: RawFd = output_file.as_raw_fd();
    let stdout_raw_fd: RawFd = io::stdout().as_raw_fd();
    dup2(output_raw_fd, stdout_raw_fd)?;

    load_rules(Box::new(CppLoader {
        ctx: get_default_kill_context()?,
    }))?;

    execve(
        &CString::new(config.program_path.as_str())?,
        &[CString::new("")?],
        &[CString::new("")?],
    )
    .unwrap();

    Ok(())
}

pub struct ResourceLimitConfig {
    pub stack_limit: (Option<u64>, Option<u64>),
    pub as_limit: (Option<u64>, Option<u64>),
    pub cpu_limit: (Option<u64>, Option<u64>),
    pub nproc_limit: (Option<u64>, Option<u64>),
    pub fsize_limit: (Option<u64>, Option<u64>),
}

impl Default for ResourceLimitConfig {
    fn default() -> Self {
        ResourceLimitConfig {
            stack_limit: (None, None),
            as_limit: (None, None),
            cpu_limit: (None, None),
            nproc_limit: (None, None),
            fsize_limit: (None, None),
        }
    }
}

fn set_resource_limit(config: &ResourceLimitConfig) -> Result<(), Errno> {
    setrlimit(RLIMIT_STACK, config.stack_limit.0, config.stack_limit.1)?;
    setrlimit(RLIMIT_AS, config.as_limit.0, config.as_limit.1)?;
    setrlimit(RLIMIT_CPU, config.cpu_limit.0, config.cpu_limit.1)?;
    setrlimit(RLIMIT_NPROC, config.nproc_limit.0, config.nproc_limit.1)?;
    setrlimit(RLIMIT_FSIZE, config.fsize_limit.0, config.fsize_limit.1)?;

    Ok(())
}

// setrlimit(
//     RLIMIT_STACK,
//     Some(1024 * 1024 * 1024),
//     Some(1024 * 1024 * 1024),
// )?;
// setrlimit(
//     RLIMIT_AS,
//     Some(1024 * 1024 * 1024),
//     Some(1024 * 1024 * 1024),
// )?;
// setrlimit(RLIMIT_CPU, Some(1), Some(1))?;
// setrlimit(RLIMIT_NPROC, None, None)?;
// setrlimit(
//     RLIMIT_FSIZE,
//     Some(1024 * 1024 * 1024),
//     Some(1024 * 1024 * 1024),
// )?;
