use crate::run::sandbox::RawRunResultInfo;
use std::{fmt, ops::Add, time::Duration};

#[derive(Debug)]
pub struct JudgeResultInfo {
    pub verdict: JudgeVerdict,
    pub time_usage: Duration,
    pub memory_usage_bytes: i64,
    pub exit_status: i32,
    pub checker_exit_status: i32,
}

#[derive(Debug, PartialEq)]
pub enum JudgeVerdict {
    Accepted,
    WrongAnswer,
    TimeLimitExceeded,
    IdlenessLimitExceeded,
    RuntimeError,
    PartialScore,
    SystemError,
}

impl fmt::Display for JudgeVerdict {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn get_run_time(raw_info: &RawRunResultInfo) -> Duration {
    let rusage = &raw_info.resource_usage;
    let utime = rusage.user_time;
    let stime = rusage.system_time;
    utime.add(stime)
}

pub fn get_max_mem(raw_info: &RawRunResultInfo) -> i64 {
    let rusage = &raw_info.resource_usage;
    rusage.max_rss
}

pub fn check_user_result(raw_info: &RawRunResultInfo) -> Option<JudgeVerdict> {
    let exit_status = raw_info.exit_status;
    log::debug!("User program exit status: {}", exit_status);
    match exit_status {
        0 => None,
        11 => Some(JudgeVerdict::RuntimeError),
        152 => Some(JudgeVerdict::TimeLimitExceeded),
        _ => Some(JudgeVerdict::SystemError),
    }
}

pub fn check_checker_result(raw_info: &RawRunResultInfo) -> JudgeVerdict {
    // TODO: return verdict according to the checker output
    let exit_status = raw_info.exit_status;
    log::debug!("Checker program exit status: {}", exit_status);
    match exit_status {
        0 => JudgeVerdict::Accepted,
        256 => JudgeVerdict::WrongAnswer,
        _ => JudgeVerdict::SystemError,
    }
}
