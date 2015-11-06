use scheduler::context::{context_clone, context_enabled, context_exit, context_switch, Context, ContextFile};
use super::parse_path::parse_path;
use collections::Vec;
use collections::string::String;

pub fn pwd() -> Vec<String> {
    unsafe {
        if let Some(current) = Context::current() {
            parse_path(&*current.cwd.get(), Vec::new())
        } else {
            Vec::new()
        }
    }
}
