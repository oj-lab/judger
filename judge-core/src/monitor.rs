use nix::{
    sys::wait::waitpid,
    unistd::{fork, write, ForkResult},
};
use std::{
    process::Command,
    thread,
};
use libc::{wait4, rusage, c_int, WSTOPPED};
use crate::{
    killer::timeout_killer,
    utils::get_default_rusage,
};

pub fn run_judge() {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            println!(
                "Continuing execution in parent process, new child has pid: {}",
                child
            );
            waitpid(child, None).unwrap();
        }
        Ok(ForkResult::Child) => {
            // Unsafe to use `println!` (or `unwrap`) here. See Safety.
            write(libc::STDOUT_FILENO, "I'm a new child process\n".as_bytes()).ok();

            // TODO: invoke runner
            let child = Command::new("./../infinite_loop")
                .spawn()
                .expect("Failed to execute child");
            
            let pid = child.id();
            thread::spawn(move || timeout_killer(pid, 5000));

            let mut status: c_int = 0;
            let mut usage: rusage = get_default_rusage();
            unsafe {
                wait4(pid as i32, &mut status, WSTOPPED, &mut usage);
            }
            
            unsafe { libc::_exit(0) };
        }
        Err(_) => println!("Fork failed"),
    }
}

#[cfg(test)]
pub mod monitor {
    use super::*;

    #[test]
    fn test_run_judge() {
        run_judge();
    }
}
