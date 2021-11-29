use super::SeccompCtxLoader;
use seccomp::*;

pub struct CppLoader;

impl SeccompCtxLoader for CppLoader {
    fn add_rules(ctx: &mut Context) -> &Context {
        ctx
    } // add_rules
}
