use crate::error::JudgeCoreError;
use crate::sandbox::RlimitConfigs;
use crate::sandbox::Sandbox;
use crate::sandbox::SandboxExitInfo;
use libc::rusage;
use libseccomp::{ScmpAction, ScmpFilterContext, ScmpSyscall};
use nix::unistd::close;
use nix::unistd::dup2;
use serde_derive::{Deserialize, Serialize};
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};
use std::time::Duration;

use super::executor::Executor;

pub struct ExecutorSandbox {
    executor: Executor,
    input_redirect: Option<RawFd>,
    output_redirect: Option<RawFd>,

    pub sandbox: Sandbox,
}

impl ExecutorSandbox {
    pub fn new(
        executor: Executor,
        rlimit_configs: RlimitConfigs,
        input_redirect: Option<RawFd>,
        output_redirect: Option<RawFd>,
        restricted: bool,
    ) -> Result<Self, JudgeCoreError> {
        log::debug!("Create sandbox with restricted={}", restricted);
        let mut scmp_filter = match restricted {
            true => ScmpFilterContext::new_filter(ScmpAction::KillProcess)?,
            false => ScmpFilterContext::new_filter(ScmpAction::Allow)?,
        };
        if restricted {
            let white_list = DEFAULT_SCMP_WHITELIST;
            for s in white_list.iter() {
                let syscall = ScmpSyscall::from_name(s)?;
                scmp_filter.add_rule_exact(ScmpAction::Allow, syscall)?;
            }
        }

        let sandbox = Sandbox::new(Some(rlimit_configs), Some(scmp_filter))?;
        Ok(Self {
            executor,
            input_redirect,
            output_redirect,
            sandbox,
        })
    }

    pub fn wait(&self) -> Result<SandboxExitInfo, JudgeCoreError> {
        self.sandbox.wait()
    }

    /// WARNING:   
    /// Unsafe to use `println!()` (or `unwrap()`) in child process.
    /// See more in `fork()` document.
    pub fn spawn(&mut self) -> Result<i32, JudgeCoreError> {
        let before_limit = {
            let input_redirect = self.input_redirect;
            let output_redirect = self.output_redirect;
            move || {
                let stderr_raw_fd = io::stderr().as_raw_fd();
                if let Some(output_redirect) = output_redirect {
                    dup2(output_redirect, stderr_raw_fd).expect("Failed to dup2 stderr");
                } else {
                    close(io::stdin().as_raw_fd()).expect("Failed to close stdin");
                }

                let stdin_raw_fd = io::stdin().as_raw_fd();
                let stdout_raw_fd = io::stdout().as_raw_fd();
                if let Some(input_redirect) = input_redirect {
                    dup2(input_redirect, stdin_raw_fd).expect("Failed to dup2 stdin");
                } else {
                    close(stdin_raw_fd).expect("Failed to close stdin");
                }

                if let Some(output_redirect) = output_redirect {
                    dup2(output_redirect, stdout_raw_fd).expect("Failed to dup2 stdout");
                } else {
                    close(stdout_raw_fd).expect("Failed to close stdout");
                }
            }
        };

        let after_limit = {
            let executor = self.executor.clone();
            move || {
                executor.exec().expect("Failed to exec");
            }
        };

        self.sandbox.spawn(before_limit, after_limit)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rusage {
    pub user_time: Duration,
    pub system_time: Duration,
    pub max_rss: i64,
    pub page_faults: i64,
    pub involuntary_context_switches: i64,
    pub voluntary_context_switches: i64,
}

impl From<rusage> for Rusage {
    fn from(rusage: rusage) -> Self {
        Self {
            user_time: Duration::new(
                rusage.ru_utime.tv_sec as u64,
                rusage.ru_utime.tv_usec as u32 * 1000,
            ),
            system_time: Duration::new(
                rusage.ru_stime.tv_sec as u64,
                rusage.ru_stime.tv_usec as u32 * 1000,
            ),
            max_rss: rusage.ru_maxrss,
            page_faults: rusage.ru_majflt,
            involuntary_context_switches: rusage.ru_nivcsw,
            voluntary_context_switches: rusage.ru_nvcsw,
        }
    }
}

const DEFAULT_SCMP_WHITELIST: [&str; 41] = [
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
    "newfstatat",
    "getrandom",
    "set_tid_address",
    "set_robust_list",
    "rseq",
    "prlimit64",
    "futex",
    "openat",
    "getcwd",
    "gettid",
    "ioctl",
    "getdents64",
    "rt_sigaction",
    "getegid",
    "geteuid",
    "getgid",
    "getuid",
    "fcntl",
    "getpid",
    "socket",
    "dup",
    "connect",
];
