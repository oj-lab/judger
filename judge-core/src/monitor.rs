use nix::{
    sys::wait::waitpid,
    unistd::{fork, write, ForkResult},
};

pub fn run_judge() {
    match unsafe{fork()} {
        Ok(ForkResult::Parent { child, .. }) => {
            println!("Continuing execution in parent process, new child has pid: {}", child);
            waitpid(child, None).unwrap();
        }
        Ok(ForkResult::Child) => {
            // Unsafe to use `println!` (or `unwrap`) here. See Safety.
            write(libc::STDOUT_FILENO, "I'm a new child process\n".as_bytes()).ok();
            unsafe { libc::_exit(0) };
        }
        Err(_) => println!("Fork failed"),
     }
}

#[cfg(test)]
pub mod run {
    #[test]
    fn test_run_judge() {
        super::run_judge();
    }
}