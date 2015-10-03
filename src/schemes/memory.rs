use common::memory::*;

use programs::common::*;
use programs::common::resource::{Resource, ResourceType, URL, VecResource};
use programs::common::string::{String, ToString};

pub struct MemoryScheme;

impl SessionItem for MemoryScheme {
    fn scheme(&self) -> String {
        return "memory".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        let string = "Memory Used: ".to_string() + memory_used() / 1024 + " KB\n" +
                     "Memory Free: " + memory_free() / 1024 + " KB";
        return box VecResource::new(URL::from_str("memory://"),
                                    ResourceType::File,
                                    string.to_utf8());
    }
}
