use super::sandbox::{RawRunResultInfo, Sandbox};
use crate::error::JudgeCoreError;
use nix::unistd::{fork, write, ForkResult};
use serde_derive::{Deserialize, Serialize};
use std::os::unix::io::RawFd;
use std::time::Instant;

pub struct ProcessListener {
    pid: i32,
    begin_time: Instant,
    child_exit_fd: i32,
    exit_signal: u8,
}

impl ProcessListener {
    pub fn new() -> Result<Self, JudgeCoreError> {
        let pid = -1;
        let begin_time = Instant::now();
        let child_exit_fd = -1;
        let exit_signal = 0u8;
        Ok(Self {
            pid,
            begin_time,
            child_exit_fd,
            exit_signal,
        })
    }

    pub fn set_exit_fd(&mut self, exit_fd: RawFd, exit_signal: u8) {
        self.child_exit_fd = exit_fd;
        self.exit_signal = exit_signal;
    }

    fn report_exit(&self, option_run_result: Option<RawRunResultInfo>) {
        if self.child_exit_fd != -1 {
            log::info!(
                "Report child {} exit to fd {}.",
                self.pid,
                self.child_exit_fd
            );
            let msg = ProcessExitMessage {
                exit_signal: self.exit_signal,
                option_run_result,
            };
            let msg_string = serde_json::to_string(&msg).unwrap() + "\n";
            let msgbuf = msg_string.as_bytes();
            let buf = [msgbuf].concat();
            write(self.child_exit_fd, &buf).unwrap();
        }
    }

    pub fn spawn_with_sandbox(
        &mut self,
        sandbox: &mut Sandbox,
    ) -> Result<Option<()>, JudgeCoreError> {
        self.begin_time = Instant::now();

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                self.pid = child.as_raw();
                Ok(Some(()))
            }
            Ok(ForkResult::Child) => {
                let process = sandbox.spawn()?;
                // listen to the status of sandbox
                log::debug!("Wait for process {}.", process);
                let run_result = sandbox.wait()?;
                log::debug!("Process {} exit.", process);
                self.pid = sandbox.child_pid;
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
