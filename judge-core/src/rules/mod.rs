pub mod cpp_loader;

use libseccomp::{error::SeccompError, ScmpAction, ScmpFilterContext};

pub trait SeccompCtxLoader {
    fn get_default_kill_context(&self) -> Result<ScmpFilterContext, SeccompError> {
        ScmpFilterContext::new_filter(ScmpAction::KillProcess)
    }

    fn add_rules(&self, ctx: ScmpFilterContext) -> Result<ScmpFilterContext, SeccompError>;

    fn load_ctx(&self) -> Result<(), SeccompError> {
        let ctx = self.get_default_kill_context()?;
        self.add_rules(ctx)?.load()?;
        Ok(())
    }
}

pub fn load_rules(loader: Box<dyn SeccompCtxLoader>) -> Result<(), SeccompError> {
    loader.load_ctx()
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
                load_rules(Box::new(CppLoader)).unwrap();
                unsafe { libc::_exit(0) };
            }
            Err(_) => println!("Fork failed"),
        }
    }
}
