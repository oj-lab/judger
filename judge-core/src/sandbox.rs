use libseccomp::{ScmpFilterContext, ScmpSyscall, ScmpAction};
use std::process::Command;
use std::io::ErrorKind;
use libc::{setrlimit, rlimit};
use crate::error::JudgeCoreError;

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
    let syscall = ["read", "write", "exit", "brk"]; // allowed system call
    for s in syscall.iter() {
      let syscall = ScmpSyscall::from_name(s).unwrap();
      filter.add_rule(ScmpAction::Allow, syscall)?;
    }
    Ok(Self {
      filter
    })
  }

  pub fn set_limit(&self, time_limit: u64, memory_limit: u64) {
    let time_limit_sec = time_limit / 1000;
    let memory_limit_bytes = memory_limit * 1024 * 1024;
    let cpu_limit = rlimit {
      rlim_cur: time_limit_sec,
      rlim_max: std::u64::MAX,
    };
    unsafe {
        setrlimit(libc::RLIMIT_CPU, &cpu_limit);
    }
    let mem_limit = rlimit {
      rlim_cur: memory_limit_bytes,
      rlim_max: std::u64::MAX,
    };
    unsafe {
        setrlimit(libc::RLIMIT_AS, &mem_limit);
    }
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
  use super::{SandBox, Verdict};

  #[test]
  fn test_sandbox_grep() {
    let sandbox = SandBox::new().unwrap();
    sandbox.set_limit(1000, 128);
    let res = sandbox.exec("grep".to_string()).unwrap();
    assert_eq!(res, Verdict::Accepted);
  }

  #[test]
  fn test_sandbox_tle() {
    let sandbox = SandBox::new().unwrap();
    sandbox.set_limit(1000, 128);
    let res = sandbox.exec("../infinite_loop".to_string()).unwrap();
    assert_eq!(res, Verdict::TimeLimitExceeded);
  }

  #[test]
  fn test_sandbox_mle() {
    let sandbox = SandBox::new().unwrap();
    sandbox.set_limit(1000, 16);
    let res = sandbox.exec("../memory_limit".to_string()).unwrap();
    assert_eq!(res, Verdict::MemoryLimitExceeded);
  }
}
