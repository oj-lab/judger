pub mod cpp_loader;

use seccomp::*;

pub trait SeccompCtxLoader {
    fn get_default_kill_context() -> Context {
        Context::default(Action::Kill).unwrap()
    }

    fn add_rules(ctx: &mut Context) -> &Context;

    fn load_ctx() {
        let mut ctx = Self::get_default_kill_context();
        Self::add_rules(&mut ctx).load().unwrap();
    }
}
