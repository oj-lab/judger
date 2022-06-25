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

#[derive(Default)]
pub struct ResourceLimitConfig {
    pub stack_limit: Option<(u64, u64)>,
    pub as_limit: Option<(u64, u64)>,
    pub cpu_limit: Option<(u64, u64)>,
    pub nproc_limit: Option<(u64, u64)>,
    pub fsize_limit: Option<(u64, u64)>,
}

fn set_resource_limit(config: &ResourceLimitConfig) -> Result<(), Errno> {
    if let Some(stack_limit) = config.stack_limit {
        setrlimit(RLIMIT_STACK, stack_limit.0, stack_limit.1)?;
    }
    if let Some(as_limit) = config.as_limit {
        setrlimit(RLIMIT_AS, as_limit.0, as_limit.1)?;
    }
    if let Some(cpu_limit) = config.cpu_limit {
        setrlimit(RLIMIT_CPU, cpu_limit.0, cpu_limit.1)?;
    }
    if let Some(nproc_limit) = config.nproc_limit {
        setrlimit(RLIMIT_NPROC, nproc_limit.0, nproc_limit.1)?;
    }
    if let Some(fsize_limit) = config.fsize_limit {
        setrlimit(RLIMIT_FSIZE, fsize_limit.0, fsize_limit.1)?;
    }

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
