use alloc::boxed::Box;

use common::memory;
use common::string::{String, ToString};

use schemes::{KScheme, Resource, URL, VecResource};

/// A memory scheme
pub struct MemoryScheme;

impl KScheme for MemoryScheme {
    fn scheme(&self) -> String {
        "memory".to_string()
    }

    fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
        let string = "Memory Used: ".to_string() + memory::memory_used() / 1024 + " KB\n" +
                     "Memory Free: " + memory::memory_free() / 1024 + " KB";
        Some(box VecResource::new(URL::from_str("memory://"), string.to_utf8()))
    }
}
