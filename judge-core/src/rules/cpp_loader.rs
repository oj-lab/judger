use super::SeccompCtxLoader;
use libseccomp::{error::SeccompError, ScmpFilterContext};

pub struct CppLoader;

impl SeccompCtxLoader for CppLoader {
    fn add_rules(&self, ctx: ScmpFilterContext) -> Result<ScmpFilterContext, SeccompError> {
        Ok(ctx)
    } // add_rules
}

