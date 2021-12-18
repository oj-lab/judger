pub mod cpp_loader;

use libseccomp::{error::SeccompError, ScmpAction, ScmpFilterContext};

pub trait SeccompCtxLoader {
    fn get_default_kill_context() -> Result<ScmpFilterContext, SeccompError> {
        ScmpFilterContext::new_filter(ScmpAction::KillProcess)
    }

    fn add_rules(ctx: &mut ScmpFilterContext) -> Result<&ScmpFilterContext, SeccompError>;

    fn load_ctx() -> Result<(), SeccompError> {
        let mut ctx = Self::get_default_kill_context()?;
        Self::add_rules(&mut ctx)?.load()?;
        Ok(())
    }
}
