pub mod cpp_loader;

use seccomp::*;

pub trait SeccompCtxLoader {
    fn get_default_kill_context() -> Result<Context, SeccompError> {
        Context::default(Action::Kill)
    }

    fn add_rules(ctx: &mut Context) -> Result<&Context, SeccompError>;

    fn load_ctx() -> Result<(), SeccompError> {
        let mut ctx = Self::get_default_kill_context()?;
        Self::add_rules(&mut ctx)?.load()?;
        Ok(())
    }
}
