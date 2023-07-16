use crate::error::JudgeCoreError;
use libc::{c_int, rusage, wait4, WEXITSTATUS, WSTOPPED, WTERMSIG};
use libseccomp::{ScmpAction, ScmpFilterContext, ScmpSyscall};
use nix::unistd::dup2;
use nix::unistd::{fork, ForkResult};
use nix::{
    sys::resource::{
        setrlimit,
        Resource::{RLIMIT_AS, RLIMIT_CPU, RLIMIT_STACK},
    },
    unistd::close,
};
use serde_derive::{Deserialize, Serialize};
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};
use std::time::{Duration, Instant};

use super::executor::Executor;

pub struct Sandbox {
    executor: Executor,
    rlimit_configs: RlimitConfigs,
    scmp_filter: ScmpFilterContext,
    input_redirect: Option<RawFd>,
    output_redirect: Option<RawFd>,
    pub child_pid: i32,
    begin_time: Instant,
}

impl Sandbox {
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

        let child_pid = -1;
        let begin_time = Instant::now();
        Ok(Self {
            executor,
            rlimit_configs,
            scmp_filter,
            input_redirect,
            output_redirect,
            child_pid,
            begin_time,
        })
    }

    /// Currently close all `stderr` and close `stdin`/`stdout` if redirect is not set
    fn load_io(&self) -> Result<(), JudgeCoreError> {
        let stderr_raw_fd = io::stderr().as_raw_fd();
        close(stderr_raw_fd)?;

        let stdin_raw_fd = io::stdin().as_raw_fd();
        let stdout_raw_fd = io::stdout().as_raw_fd();
        if let Some(input_redirect) = self.input_redirect {
            dup2(input_redirect, stdin_raw_fd)?;
        } else {
            close(stdin_raw_fd)?;
        }

        if let Some(output_redirect) = self.output_redirect {
            dup2(output_redirect, stdout_raw_fd)?;
        } else {
            close(stdout_raw_fd)?;
        }
        Ok(())
    }

    pub fn wait(&self) -> Result<RawRunResultInfo, JudgeCoreError> {
        let mut status: c_int = 0;
        let mut usage: rusage = get_default_rusage();
        unsafe {
            wait4(self.child_pid, &mut status, WSTOPPED, &mut usage);
        }

        log::info!("Detected process pid={} exit", self.child_pid);

        Ok(RawRunResultInfo {
            exit_status: status,
            exit_signal: WTERMSIG(status),
            exit_code: WEXITSTATUS(status),
            real_time_cost: self.begin_time.elapsed(),
            resource_usage: Rusage::from(usage),
        })
    }

    /// WARNING:   
    /// Unsafe to use `println!()` (or `unwrap()`) in child process.
    /// See more in `fork()` document.
    pub fn spawn(&mut self) -> Result<i32, JudgeCoreError> {
        let now = Instant::now();
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                log::info!("Forked child pid={}", child);
                self.child_pid = child.as_raw();
                self.begin_time = now;
                Ok(child.as_raw())
            }
            // child process should not return to do things outside `spawn()`
            Ok(ForkResult::Child) => {
                // TODO: maybe customed error handler are needed
                self.load_io().expect("Failed to load io redirect");
                self.rlimit_configs
                    .load()
                    .expect("Failed to load rlimit configs");
                self.scmp_filter
                    .load()
                    .expect("Failed to load seccomp filter");

                self.executor.exec().expect("Failed to exec");
                unsafe { libc::_exit(0) };
            }
            Err(e) => Err(JudgeCoreError::NixErrno(e)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawRunResultInfo {
    pub exit_status: c_int,
    pub exit_signal: c_int,
    pub exit_code: c_int,
    pub real_time_cost: Duration,
    pub resource_usage: Rusage,
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

#[derive(Default, Debug, Clone)]
pub struct RlimitConfigs {
    pub stack_limit: Option<(u64, u64)>,
    pub as_limit: Option<(u64, u64)>,
    pub cpu_limit: Option<(u64, u64)>,
    pub nproc_limit: Option<(u64, u64)>,
    pub fsize_limit: Option<(u64, u64)>,
}

impl RlimitConfigs {
    pub fn load(&self) -> Result<(), JudgeCoreError> {
        if let Some(stack_limit) = self.stack_limit {
            log::debug!("Set stack limit: {:?}", stack_limit);
            setrlimit(RLIMIT_STACK, stack_limit.0, stack_limit.1)?;
        }
        if let Some(as_limit) = self.as_limit {
            log::debug!("Set as limit: {:?}", as_limit);
            setrlimit(RLIMIT_AS, as_limit.0, as_limit.1)?;
        }
        if let Some(cpu_limit) = self.cpu_limit {
            log::debug!("Set cpu limit: {:?}", cpu_limit);
            setrlimit(RLIMIT_CPU, cpu_limit.0, cpu_limit.1)?;
        }
        Ok(())
    }
}

pub const SCRIPT_LIMIT_CONFIG: RlimitConfigs = RlimitConfigs {
    stack_limit: Some((16 * 1024 * 1024, 16 * 1024 * 1024)),
    as_limit: Some((1024 * 1024 * 1024, 1024 * 1024 * 1024)),
    cpu_limit: Some((60, 90)),
    nproc_limit: Some((1, 1)),
    fsize_limit: Some((1024, 1024)),
};

fn get_default_rusage() -> rusage {
    rusage {
        ru_utime: libc::timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        ru_stime: libc::timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        ru_maxrss: 0,
        ru_ixrss: 0,
        ru_idrss: 0,
        ru_isrss: 0,
        ru_minflt: 0,
        ru_majflt: 0,
        ru_nswap: 0,
        ru_inblock: 0,
        ru_oublock: 0,
        ru_msgsnd: 0,
        ru_msgrcv: 0,
        ru_nsignals: 0,
        ru_nvcsw: 0,
        ru_nivcsw: 0,
    }
}

const DEFAULT_SCMP_WHITELIST: [&str; 27] = [
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
];
