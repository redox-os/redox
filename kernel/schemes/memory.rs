use alloc::boxed::Box;

use collections::string::{String, ToString};

use common::memory;

use schemes::{KScheme, Resource, URL, VecResource};

/// A memory scheme
pub struct MemoryScheme;

impl KScheme for MemoryScheme {
    fn scheme(&self) -> String {
        "memory".to_string()
    }

    fn open(&mut self, url: &URL) -> Option<Box<Resource>> {
        let string = format!("Memory Used: {} KB\nMemory Free: {} KB", memory::memory_used() / 1024, memory::memory_free() / 1024);
        Some(box VecResource::new(URL::from_str("memory://"), string.into_bytes()))
    }
}
