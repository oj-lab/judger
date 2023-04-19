use crate::error::JudgeCoreError;
use libseccomp::{ScmpAction, ScmpFilterContext, ScmpSyscall};
use nix::sys::resource::{
    setrlimit,
    Resource::{RLIMIT_AS, RLIMIT_CPU, RLIMIT_STACK},
};
use nix::unistd::{dup2, execve};
use std::ffi::CString;
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};

#[derive(Default)]
pub struct ResourceLimitConfig {
    pub stack_limit: Option<(u64, u64)>,
    pub as_limit: Option<(u64, u64)>,
    pub cpu_limit: Option<(u64, u64)>,
    pub nproc_limit: Option<(u64, u64)>,
    pub fsize_limit: Option<(u64, u64)>,
}

pub struct SandBox {
    filter: ScmpFilterContext,
    stdin_raw_fd: RawFd,
    stdout_raw_fd: RawFd,
}

impl SandBox {
    pub fn new() -> Result<Self, JudgeCoreError> {
        let mut filter = ScmpFilterContext::new_filter(ScmpAction::KillProcess).unwrap();
        let white_list: Vec<&str> = vec![
            "read",
            "fstat",
            "mmap",
            "mprotect",
            "munmap",
            "uname",
            "arch_prctl",
            "brk",
            "access",
            "exit_group",
            "close",
            "readlink",
            "sysinfo",
            "write",
            "writev",
            "lseek",
            "clock_gettime",
            "pread64",
            "execve",
            "open",
            "openat",
        ];
        for s in white_list.iter() {
            let syscall = ScmpSyscall::from_name(s)?;
            filter.add_rule_exact(ScmpAction::Allow, syscall)?;
        }
        let stdin_raw_fd = io::stdin().as_raw_fd();
        let stdout_raw_fd = io::stdout().as_raw_fd();
        Ok(Self {
            filter,
            stdin_raw_fd,
            stdout_raw_fd,
        })
    }

    pub fn set_io(&self, input_raw_fd: RawFd, output_raw_fd: RawFd) {
        dup2(input_raw_fd, self.stdin_raw_fd).unwrap();
        dup2(output_raw_fd, self.stdout_raw_fd).unwrap();
    }

    pub fn set_limit(&self, config: &ResourceLimitConfig) -> Result<(), JudgeCoreError> {
        if let Some(stack_limit) = config.stack_limit {
            setrlimit(RLIMIT_STACK, stack_limit.0, stack_limit.1)?;
        }
        if let Some(as_limit) = config.as_limit {
            setrlimit(RLIMIT_AS, as_limit.0, as_limit.1)?;
        }
        if let Some(cpu_limit) = config.cpu_limit {
            setrlimit(RLIMIT_CPU, cpu_limit.0, cpu_limit.1)?;
        }
        Ok(())
    }

    pub fn exec(&self, command: &str) -> Result<(), JudgeCoreError> {
        self.filter.load()?;
        println!("start to execve");
        execve(
            &CString::new(command)?,
            &[CString::new("")?],
            &[CString::new("")?],
        )
        .unwrap();
        Ok(())
    }
}
