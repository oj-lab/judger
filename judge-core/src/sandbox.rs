use libseccomp::{ScmpFilterContext, ScmpSyscall, ScmpAction};
use std::process::Command;
use std::io::ErrorKind;
use nix::{
  sys::resource::{
      setrlimit,
      Resource::{RLIMIT_AS, RLIMIT_CPU, RLIMIT_STACK},
  }
};
use crate::error::JudgeCoreError;

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
}

#[derive(PartialEq, Debug)]
pub enum Verdict {
    Accepted,
    WrongAnswer,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    RuntimeError
}

impl SandBox {
  pub fn new() -> Result<Self, JudgeCoreError> {
    let mut filter = ScmpFilterContext::new_filter(ScmpAction::Allow).unwrap();
    // for s in white_list.iter() {
    //   let syscall = ScmpSyscall::from_name(s).unwrap();
    //   filter.add_rule_exact(ScmpAction::Allow, syscall)?;
    // }
    Ok(Self {
      filter
    })
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

  pub fn exec(&self, command: String) -> Result<Verdict, JudgeCoreError> {
    self.filter.load()?;
    match Command::new("sh")
      .arg("-c")
      .arg(&command)
      .spawn() {
        Ok(res) => {
          let output = res.wait_with_output().unwrap();
          let status = output.status;
          match status.code() {
            Some(0) => {
              Ok(Verdict::Accepted)
            },
            Some(139) => Ok(Verdict::RuntimeError),
            Some(152) => Ok(Verdict::TimeLimitExceeded),
            _ => panic!("Unexpected status {}", status)
          }
        },
        Err(e) => match e.kind() {
          ErrorKind::OutOfMemory => Ok(Verdict::MemoryLimitExceeded),
          _ => Err(JudgeCoreError::from(e)),
        },
      }
  }
}

#[cfg(test)]
pub mod sandbox {
  use super::{SandBox, Verdict, ResourceLimitConfig};

  const TEST_CONFIG: ResourceLimitConfig = ResourceLimitConfig {
    stack_limit: Some((64 * 1024 * 1024, 64 * 1024 * 1024)),
    as_limit: Some((128 * 1024 * 1024, 128 * 1024 * 1024)),
    cpu_limit: Some((1, 2)),
    nproc_limit: Some((1, 1)),
    fsize_limit: Some((1024, 1024)),
  };

  #[test]
  fn test_sandbox_ls() {
    let sandbox = SandBox::new().unwrap();
    sandbox.set_limit(&TEST_CONFIG);
    let res = sandbox.exec("ls".to_string()).unwrap();
    assert_eq!(res, Verdict::Accepted);
  }

  #[test]
  fn test_sandbox_tle() {
    let sandbox = SandBox::new().unwrap();
    sandbox.set_limit(&TEST_CONFIG);
    let res = sandbox.exec("../infinite_loop".to_string()).unwrap();
    assert_eq!(res, Verdict::TimeLimitExceeded);
  }

  #[test]
  fn test_sandbox_mle() {
    let sandbox = SandBox::new().unwrap();
    sandbox.set_limit(&TEST_CONFIG);
    let res = sandbox.exec("../memory_limit".to_string()).unwrap();
    assert_eq!(res, Verdict::RuntimeError);
  }
}
