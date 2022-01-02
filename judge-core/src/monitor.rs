use crate::{killer::timeout_killer, runner::run_process};

use nix::{
    sys::wait::waitpid,
    unistd::{fork, write, ForkResult},
};
use std::thread;

pub fn run_judge() {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            println!(
                "Continuing execution in parent process, new child has pid: {}",
                child
            );

            thread::spawn(move || timeout_killer(child.as_raw() as u32, 5000));
            println!("timeout_killer has been set.");

            waitpid(child, None).unwrap();
        }
        Ok(ForkResult::Child) => {
            // Unsafe to use `println!` (or `unwrap`) here. See Safety.
            write(libc::STDOUT_FILENO, "I'm a new child process\n".as_bytes()).ok();

            run_process();

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
