use super::sandbox::{RawRunResultInfo, Sandbox};
use crate::error::JudgeCoreError;
use nix::unistd::{fork, write, ForkResult};
use serde_derive::{Deserialize, Serialize};
use std::os::unix::io::RawFd;

pub struct ProcessListener {
    child_exit_fd: i32,
    exit_signal: u8,
}

impl ProcessListener {
    pub fn new() -> Result<Self, JudgeCoreError> {
        let child_exit_fd = -1;
        let exit_signal = 0u8;
        Ok(Self {
            child_exit_fd,
            exit_signal,
        })
    }

    pub fn setup_exit_report(&mut self, exit_fd: RawFd, exit_signal: u8) {
        self.child_exit_fd = exit_fd;
        self.exit_signal = exit_signal;
    }

    fn report_exit(&self, option_run_result: Option<RawRunResultInfo>) {
        if self.child_exit_fd != -1 {
            let msg = ProcessExitMessage {
                exit_signal: self.exit_signal,
                option_run_result,
            };
            let buf = serde_json::to_vec(&msg).expect("Serialize failed.");
            write(self.child_exit_fd, &buf).unwrap();
        }
    }

    pub fn spawn_with_sandbox(
        &mut self,
        sandbox: &mut Sandbox,
    ) -> Result<Option<()>, JudgeCoreError> {
        match unsafe { fork() } {
            Ok(ForkResult::Parent { .. }) => Ok(Some(())),
            Ok(ForkResult::Child) => {
                let process = sandbox.spawn()?;
                // listen to the status of sandbox
                log::debug!("Wait for process {}.", process);
                let run_result = sandbox.wait()?;
                log::debug!("Process {} exit.", process);
                self.report_exit(Some(run_result));
                unsafe { libc::_exit(0) };
            }
            Err(_) => {
                panic!("Fork failed.");
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessExitMessage {
    pub exit_signal: u8,
    pub option_run_result: Option<RawRunResultInfo>,
}
