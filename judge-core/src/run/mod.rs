use nix::sys::resource::{
    setrlimit,
    Resource::{RLIMIT_AS, RLIMIT_CPU, RLIMIT_STACK},
};
use serde_derive::Serialize;

use crate::error::JudgeCoreError;

pub mod executor;
pub mod process_listener;
pub mod sandbox;

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
