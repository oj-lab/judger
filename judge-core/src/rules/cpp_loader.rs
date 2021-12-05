use super::SeccompCtxLoader;
use seccomp::*;

pub struct CppLoader;

impl SeccompCtxLoader for CppLoader {
    fn add_rules(ctx: &mut Context) -> Result<&Context, SeccompError> {
        Ok(ctx)
    } // add_rules
}
