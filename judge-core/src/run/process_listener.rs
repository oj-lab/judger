use super::sandbox::Sandbox;
use crate::error::JudgeCoreError;
use nix::unistd::{fork, write, ForkResult};
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

    fn report_exit(&self) {
        if self.child_exit_fd != -1 {
            log::info!(
                "Report child {} exit to fd {}.",
                self.pid,
                self.child_exit_fd
            );
            let buf = [self.exit_signal];
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
                log::debug!("Child process {} start.", self.pid);
                let process = sandbox.spawn()?;
                if process.is_some() {
                    // listen to the status of sandbox
                    log::debug!("Wait for process {}.", self.pid);
                    let _result = sandbox.wait()?;
                    self.pid = sandbox.child_pid;
                    self.report_exit();
                    // how to send the result to parent???
                }
                Ok(None)
            }
            Err(_) => {
                panic!("Fork failed.");
            }
        }
    }
}
