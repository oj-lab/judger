use crate::{killer::timeout_killer, runner::run_process, utils::get_default_rusage};
use libc::{c_int, rusage, wait4, WSTOPPED};
use nix::unistd::{fork, write, ForkResult};
use std::thread;

pub fn run_judge() -> Option<(c_int, rusage)> {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            println!(
                "Continuing execution in parent process, new child has pid: {}",
                child
            );

            thread::spawn(move || timeout_killer(child.as_raw() as u32, 5000));
            println!("timeout_killer has been set");

            let mut status: c_int = 0;
            let mut usage: rusage = get_default_rusage();
            unsafe {
                wait4(child.as_raw() as i32, &mut status, WSTOPPED, &mut usage);
            }

            println!("Detected process exit");

            Some((status, usage))
        }
        Ok(ForkResult::Child) => {
            // Unsafe to use `println!` (or `unwrap`) here. See Safety.
            write(libc::STDOUT_FILENO, "I'm a new child process\n".as_bytes()).ok();

            run_process();

            None
        }
        Err(_) => {
            println!("Fork failed");

            None
        }
    }
}

#[cfg(test)]
pub mod monitor {
    use super::*;

    #[test]
    fn test_run_judge() {
        let result = run_judge();
        println!("{:?}", result.unwrap().0);
        println!("{:?}", result.unwrap().1);
    }
}
