use common::memory::*;
use common::string::*;
use common::url::*;

use programs::session::*;

pub struct MemoryScheme;

impl SessionScheme for MemoryScheme {
    fn scheme(&self) -> String {
        return "memory".to_string();
    }

    #[allow(unused_variables)]
    fn on_url(&mut self, session: &Session, url: &URL) -> String {
        return String::new() + "Memory Used: " + memory_used()/1024/1024 + " MB\n" + "Memory Free: " + memory_free()/1024/1024 + " MB";
    }
}
