use libseccomp::{ScmpFilterContext, ScmpSyscall, ScmpAction};
use std::process::Command;
use std::io::ErrorKind;
use nix::{
  sys::resource::{
      setrlimit,
      Resource::{RLIMIT_AS, RLIMIT_CPU, RLIMIT_FSIZE, RLIMIT_NPROC, RLIMIT_STACK},
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
    let syscall = vec!["read", "write", "exit", "brk"]; // allowed system call
    for s in syscall.iter() {
      let syscall = ScmpSyscall::from_name(s).unwrap();
      println!("try add syscall: {} {}", s, syscall);
      filter.add_rule(ScmpAction::Allow, syscall)?;
    }
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
    if let Some(nproc_limit) = config.nproc_limit {
      setrlimit(RLIMIT_NPROC, nproc_limit.0, nproc_limit.1)?;
    }
    if let Some(fsize_limit) = config.fsize_limit {
      setrlimit(RLIMIT_FSIZE, fsize_limit.0, fsize_limit.1)?;
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
            Some(152) => Ok(Verdict::TimeLimitExceeded),
            _ => panic!("Unexpected error")
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

  #[test]
  fn test_sandbox_grep() {
    let sandbox = SandBox::new().unwrap();
    sandbox.set_limit(&ResourceLimitConfig {cpu_limit: Some((1, 2)), as_limit: Some((128, 128)), ..Default::default()});
    let res = sandbox.exec("grep".to_string()).unwrap();
    assert_eq!(res, Verdict::Accepted);
  }

  #[test]
  fn test_sandbox_tle() {
    let sandbox = SandBox::new().unwrap();
    sandbox.set_limit(&ResourceLimitConfig {cpu_limit: Some((1, 2)), as_limit: Some((128, 128)), ..Default::default()});
    let res = sandbox.exec("../infinite_loop".to_string()).unwrap();
    assert_eq!(res, Verdict::TimeLimitExceeded);
  }

  #[test]
  fn test_sandbox_mle() {
    let sandbox = SandBox::new().unwrap();
    sandbox.set_limit(&ResourceLimitConfig {cpu_limit: Some((1, 2)), as_limit: Some((16, 16)), ..Default::default()});
    let res = sandbox.exec("../memory_limit".to_string()).unwrap();
    assert_eq!(res, Verdict::MemoryLimitExceeded);
  }
}
