pub mod cpp_loader;

use libseccomp::{error::SeccompError, ScmpAction, ScmpFilterContext};

pub fn get_default_kill_context() -> Result<ScmpFilterContext, SeccompError> {
    ScmpFilterContext::new_filter(ScmpAction::KillProcess)
}

pub trait SeccompCtxLoader {
    fn add_rules(&mut self) -> Result<(), SeccompError>;

    fn load_ctx(&self) -> Result<(), SeccompError>;
}

pub fn load_rules(mut loader: Box<dyn SeccompCtxLoader>) -> Result<(), SeccompError> {
    loader.add_rules()?;
    loader.load_ctx()?;
    Ok(())
}

#[cfg(test)]
pub mod rules {
    use nix::{
        sys::wait::waitpid,
        unistd::{fork, write, ForkResult},
    };

    #[test]
    pub fn test_cpp_loader() {
        use super::cpp_loader::CppLoader;
        use super::*;

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
                load_rules(Box::new(CppLoader {
                    ctx: get_default_kill_context().unwrap(),
                }))
                .unwrap();
                unsafe { libc::_exit(0) };
            }
            Err(_) => println!("Fork failed"),
        }
    }
}
