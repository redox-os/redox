use alloc::boxed::*;

use common::memory::*;
use common::string::*;
use common::url::*;

use programs::session::*;

pub struct MemoryScheme;

impl SessionModule for MemoryScheme {
    fn scheme(&self) -> String {
        return "memory".to_string();
    }

    #[allow(unused_variables)]
    fn request(&mut self, session: &Session, url: &URL, callback: Box<FnBox(String)>){
        callback("Memory Used: ".to_string() + memory_used()/1024/1024 + " MB\n" + "Memory Free: " + memory_free()/1024/1024 + " MB");
    }
}
