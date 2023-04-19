use crate::{sandbox::RawRunResultInfo, monitor::RunnerConfig};
use std::collections::HashSet;

#[derive(Debug)]
pub struct JudgeResultInfo {
    pub raw: RawRunResultInfo,
    pub problems: HashSet<JudgeProblemType>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum JudgeProblemType {
    SystemError,
    RuntimeError,
    MemoryLimitExceeded,
    RealTimeLimitExceeded,
    CpuTimeLimitExceeded,
}

pub fn infer_result(raw_info: RawRunResultInfo, _runner_config: RunnerConfig) -> JudgeResultInfo {
    let mut problems = HashSet::new();

    // TODO: Fullfill problem infer
    if raw_info.exit_signal == libc::SIGUSR1 {
        problems.insert(JudgeProblemType::SystemError);
    } else if raw_info.exit_code != 0 {
        problems.insert(JudgeProblemType::RuntimeError);
    }

    JudgeResultInfo {
        raw: raw_info,
        problems,
    }
}
