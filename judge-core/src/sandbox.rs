use crate::{error::JudgeCoreError, utils::get_default_rusage};
use libc::{c_int, rusage, wait4, WEXITSTATUS, WSTOPPED, WTERMSIG, waitpid};
use libseccomp::{ScmpAction, ScmpFilterContext, ScmpSyscall};
use nix::sys::resource::{
    setrlimit,
    Resource::{RLIMIT_AS, RLIMIT_CPU, RLIMIT_STACK},
};
use nix::unistd::{dup2, execve};
use nix::unistd::{fork, write, ForkResult};
use std::ffi::CString;
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct ResourceLimitConfig {
    pub stack_limit: Option<(u64, u64)>,
    pub as_limit: Option<(u64, u64)>,
    pub cpu_limit: Option<(u64, u64)>,
    pub nproc_limit: Option<(u64, u64)>,
    pub fsize_limit: Option<(u64, u64)>,
}

#[derive(Debug)]
pub struct RawRunResultInfo {
    pub exit_status: c_int,
    pub exit_signal: c_int,
    pub exit_code: c_int,
    pub real_time_cost: Duration,
    pub resource_usage: rusage,
}

pub struct SandBox {
    filter: ScmpFilterContext,
    stdin_raw_fd: RawFd,
    stdout_raw_fd: RawFd,
    pub child_pid: i32,
    begin_time: Instant,
}

impl SandBox {
    pub fn new(restricted: bool) -> Result<Self, JudgeCoreError> {
        let mut filter = match restricted {
            true => ScmpFilterContext::new_filter(ScmpAction::KillProcess).unwrap(),
            false => ScmpFilterContext::new_filter(ScmpAction::Allow).unwrap(),
        };
        if restricted {
            let white_list: Vec<&str> = vec![
                "read",
                "fstat",
                "mmap",
                "mprotect",
                "munmap",
                "uname",
                "arch_prctl",
                "brk",
                "access",
                "exit_group",
                "close",
                "readlink",
                "sysinfo",
                "write",
                "writev",
                "lseek",
                "clock_gettime",
                "pread64",
                "execve",
                "open",
                "openat",
            ];
            for s in white_list.iter() {
                let syscall = ScmpSyscall::from_name(s)?;
                filter.add_rule_exact(ScmpAction::Allow, syscall)?;
            }
        }
        let stdin_raw_fd = io::stdin().as_raw_fd();
        let stdout_raw_fd = io::stdout().as_raw_fd();
        let child_pid = -1;
        let begin_time = Instant::now();
        Ok(Self {
            filter,
            stdin_raw_fd,
            stdout_raw_fd,
            child_pid,
            begin_time,
        })
    }

    pub fn set_io(&self, input_raw_fd: RawFd, output_raw_fd: RawFd) {
        println!("process {}: Set up io in: {}, out: {}", self.child_pid, input_raw_fd, output_raw_fd);
        dup2(input_raw_fd, self.stdin_raw_fd).unwrap();
        dup2(output_raw_fd, self.stdout_raw_fd).unwrap();
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

    pub fn wait(
        &self,
    ) -> Result<Option<RawRunResultInfo>, JudgeCoreError> {
        let mut status: c_int = 0;
        let mut usage: rusage = get_default_rusage();
        unsafe {
            wait4(self.child_pid, &mut status, WSTOPPED, &mut usage);
        }

        println!("Detected process exit");

        Ok(Some(RawRunResultInfo {
            exit_status: status,
            exit_signal: WTERMSIG(status),
            exit_code: WEXITSTATUS(status),
            real_time_cost: self.begin_time.elapsed(),
            resource_usage: usage,
        }))
    }

    pub fn spawn(
        &mut self,
        runner_cmd: &String,
        runner_args: &Vec<&String>,
        rlimit_config: &ResourceLimitConfig,
    ) -> Result<Option<()>, JudgeCoreError> {
        let now = Instant::now();

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                println!(
                    "Continuing execution in parent process, new child has pid: {}",
                    child
                );

                self.child_pid = child.as_raw();
                self.begin_time = now;
                
                Ok(Some(()))
            }
            Ok(ForkResult::Child) => {
                // Unsafe to use `println!` (or `unwrap`) here. See Safety.

                self.set_limit(&rlimit_config)?;
                self.exec(&runner_cmd, runner_args).unwrap();

                Ok(None)
            }
            Err(_) => {
                println!("Fork failed");

                Ok(None)
            }
        }
    }

    pub fn spawn_with_io(
        &mut self,
        runner_cmd: &String,
        runner_args: &Vec<&String>,
        rlimit_config: &ResourceLimitConfig,
        input_raw_fd: RawFd,
        output_raw_fd: RawFd,
    ) -> Result<Option<()>, JudgeCoreError> {
        let now = Instant::now();

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                println!(
                    "Continuing execution in parent process, new child has pid: {}",
                    child
                );

                self.child_pid = child.as_raw();
                self.begin_time = now;

                Ok(Some(()))
            }
            Ok(ForkResult::Child) => {
                // Unsafe to use `println!` (or `unwrap`) here. See Safety.
                write(libc::STDOUT_FILENO, "I'm a new child process\n".as_bytes()).ok();
                
                self.set_io(input_raw_fd, output_raw_fd);
                self.set_limit(&rlimit_config)?;
                self.exec(&runner_cmd, runner_args).unwrap();

                Ok(None)
            }
            Err(_) => {
                println!("Fork failed");

                Ok(None)
            }
        }
    }

    pub fn exec(&self, command: &str, args: &Vec<&String>) -> Result<(), JudgeCoreError> {
        self.filter.load()?;
        let c_args = args
            .iter()
            .map(|s| CString::new(s.as_bytes()))
            .collect::<Result<Vec<_>, _>>()?;
        execve(
            &CString::new(command)?,
            &c_args.as_slice(),
            &[CString::new("")?],
        )
        .unwrap();
        Ok(())
    }
}


pub struct ProcessListener {
    sandbox: SandBox,
    pid: i32,
    begin_time: Instant,
    child_exit_fd: i32,
    exit_signal: u8,
}

impl ProcessListener {
    pub fn new(restricted: bool) -> Result<Self, JudgeCoreError> {
        let sandbox = SandBox::new(restricted)?;
        let pid = -1;
        let begin_time = Instant::now();
        let child_exit_fd = -1;
        let exit_signal = 0u8;
        Ok(Self {
            sandbox,
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
            println!("Report child {} exit to fd {}.", self.pid, self.child_exit_fd);
            let buf = [self.exit_signal];
            write(self.child_exit_fd, &buf).unwrap();
        }
    }

    pub fn spawn(&mut self,
        runner_cmd: &String,
        runner_args: &Vec<&String>,
        rlimit_config: &ResourceLimitConfig,
    ) -> Result<Option<()>, JudgeCoreError> {
        self.begin_time = Instant::now();

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                self.pid = child.as_raw();
                Ok(Some(()))
            }
            Ok(ForkResult::Child) => {
                let process = self.sandbox.spawn(runner_cmd, runner_args, rlimit_config)?;
                if !process.is_none() {
                    // listen to the status of sandbox
                    let result = self.sandbox.wait()?;
                    // how to send the result to parent???
                }
                Ok(None)
            }
            Err(_) => {
                panic!("Fork failed.");
            }
        }
    }

    pub fn spawn_with_io(&mut self,
        runner_cmd: &String,
        runner_args: &Vec<&String>,
        rlimit_config: &ResourceLimitConfig,
        input_raw_fd: RawFd,
        output_raw_fd: RawFd,
    ) -> Result<Option<()>, JudgeCoreError> {
        self.begin_time = Instant::now();

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                self.pid = child.as_raw();
                Ok(Some(()))
            }
            Ok(ForkResult::Child) => {
                let process = self.sandbox.spawn_with_io(runner_cmd, runner_args, rlimit_config, input_raw_fd, output_raw_fd)?;
                if !process.is_none() {
                    // listen to the status of sandbox
                    let result = self.sandbox.wait()?;
                    self.pid = self.sandbox.child_pid;
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
