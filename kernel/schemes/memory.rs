use alloc::boxed::Box;

use common::memory;

use schemes::{KScheme, Resource, Url, VecResource};

/// A memory scheme
pub struct MemoryScheme;

impl KScheme for MemoryScheme {
    fn scheme(&self) -> &str {
        "memory"
    }

    fn open(&mut self, _: &Url, _: usize) -> Option<Box<Resource>> {
        let string = format!("Memory Used: {} KB\nMemory Free: {} KB",
                             memory::memory_used() / 1024,
                             memory::memory_free() / 1024);
        Some(box VecResource::new(Url::from_str("memory:"), string.into_bytes()))
    }
}
