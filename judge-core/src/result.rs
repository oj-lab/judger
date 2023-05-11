use crate::sandbox::RawRunResultInfo;

#[derive(Debug)]
pub struct JudgeResultInfo {
    pub verdict: JudgeVerdict,
    pub time: i64,
    pub memory: i64,
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

pub fn get_run_time(raw_info: &RawRunResultInfo) -> i64 {
    let rusage = raw_info.resource_usage;
    let utime = rusage.ru_utime.tv_sec * 1000 + rusage.ru_utime.tv_usec / 1000;
    let stime = rusage.ru_utime.tv_sec * 1000 + rusage.ru_utime.tv_usec / 1000;
    utime + stime
}

pub fn get_max_mem(raw_info: &RawRunResultInfo) -> i64 {
    let rusage = raw_info.resource_usage;
    rusage.ru_maxrss
}

pub fn check_user_result(raw_info: &RawRunResultInfo) -> Option<JudgeVerdict> {
    let exit_status = raw_info.exit_status;
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
    match exit_status {
        0 => JudgeVerdict::Accepted,
        256 => JudgeVerdict::WrongAnswer,
        _ => JudgeVerdict::SystemError,
    }
}
