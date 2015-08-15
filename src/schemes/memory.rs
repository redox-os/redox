use alloc::boxed::*;

use common::memory::*;
use common::resource::*;
use common::string::*;

use programs::session::*;

pub struct MemoryScheme;

impl SessionModule for MemoryScheme {
    fn scheme(&self) -> String {
        return "memory".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource>{
        let string = "Memory Used: ".to_string() + memory_used()/1024/1024 + " MB\n" + "Memory Free: " + memory_free()/1024/1024 + " MB";
        return box VecResource::new(string.to_utf8());
    }
}
