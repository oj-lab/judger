use crate::error::JudgeCoreError;
use libc::{c_int, rusage, wait4, WEXITSTATUS, WSTOPPED, WTERMSIG};
use libseccomp::ScmpFilterContext;
use nix::sys::resource::{
    setrlimit,
    Resource::{RLIMIT_AS, RLIMIT_CPU, RLIMIT_STACK},
};
use nix::unistd::{fork, ForkResult};
use serde_derive::{Deserialize, Serialize};
use std::time::{Duration, Instant};

pub static DEFAULT_RLIMIT_CONFIGS: RlimitConfigs = RlimitConfigs {
    stack_limit: Some((64 * 1024 * 1024, 64 * 1024 * 1024)),
    as_limit: Some((64 * 1024 * 1024, 64 * 1024 * 1024)),
    cpu_limit: Some((1, 2)),
    nproc_limit: Some((1, 1)),
    fsize_limit: Some((1024, 1024)),
};

pub static SCRIPT_LIMIT_CONFIG: RlimitConfigs = RlimitConfigs {
    stack_limit: Some((16 * 1024 * 1024, 16 * 1024 * 1024)),
    as_limit: Some((1024 * 1024 * 1024, 1024 * 1024 * 1024)),
    cpu_limit: Some((60, 90)),
    nproc_limit: Some((1, 1)),
    fsize_limit: Some((1024, 1024)),
};

#[derive(Default, Debug, Clone, Serialize)]
pub struct RlimitConfigs {
    pub stack_limit: Option<(u64, u64)>,
    pub as_limit: Option<(u64, u64)>,
    pub cpu_limit: Option<(u64, u64)>,
    pub nproc_limit: Option<(u64, u64)>,
    pub fsize_limit: Option<(u64, u64)>,
}

impl RlimitConfigs {
    /// Load the rlimit configs to the current process.
    ///
    /// One thing should be noted is that `RLIMIT_CPU` is set to +1 second of the given value.
    /// This is because rlimit will kills the process when CPU almost reaches the limit,
    /// which can have a few milliseconds of deviation.
    pub fn load(&self) -> Result<(), JudgeCoreError> {
        if let Some(stack_limit) = self.stack_limit {
            setrlimit(RLIMIT_STACK, stack_limit.0, stack_limit.1)?;
        }
        if let Some(as_limit) = self.as_limit {
            setrlimit(RLIMIT_AS, as_limit.0, as_limit.1)?;
        }
        if let Some(cpu_limit) = self.cpu_limit {
            setrlimit(RLIMIT_CPU, cpu_limit.0 + 1, cpu_limit.1 + 1)?;
        }
        Ok(())
    }

    pub fn get_cpu_limit_duration(&self) -> Option<std::time::Duration> {
        self.cpu_limit
            .map(|(soft, _)| std::time::Duration::from_secs(soft))
    }
}

pub struct Sandbox {
    pub child_pid: i32,

    rlimit_configs: Option<RlimitConfigs>,
    scmp_filter: Option<ScmpFilterContext>,

    begin_time: Instant,
}

impl Sandbox {
    pub fn new(
        rlimit_configs: Option<RlimitConfigs>,
        scmp_filter: Option<ScmpFilterContext>,
    ) -> Result<Self, JudgeCoreError> {
        let child_pid = -1;
        let begin_time = Instant::now();
        Ok(Self {
            rlimit_configs,
            scmp_filter,
            child_pid,
            begin_time,
        })
    }

    pub fn wait(&self) -> Result<SandboxExitInfo, JudgeCoreError> {
        let mut status: c_int = 0;
        let mut usage: rusage = get_default_rusage();
        unsafe {
            wait4(self.child_pid, &mut status, WSTOPPED, &mut usage);
        }

        log::info!("Detected process pid={} exit", self.child_pid);

        Ok(SandboxExitInfo {
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
    pub fn spawn(
        &mut self,
        before_limit: impl Fn(),
        after_limit: impl Fn(),
    ) -> Result<i32, JudgeCoreError> {
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
                before_limit();
                if let Some(rlimit_configs) = &self.rlimit_configs {
                    rlimit_configs
                        .load()
                        .expect("Failed to load rlimit configs");
                }
                if let Some(scmp_filter) = &self.scmp_filter {
                    scmp_filter.load().expect("Failed to load seccomp filter");
                }
                after_limit();
                unsafe { libc::_exit(0) };
            }
            Err(e) => Err(JudgeCoreError::NixErrno(e)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SandboxExitInfo {
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
