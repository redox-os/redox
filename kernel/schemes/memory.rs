use alloc::boxed::Box;

use arch::memory;

use schemes::{Result, KScheme, Resource, Url, VecResource};

/// A memory scheme
pub struct MemoryScheme;

impl KScheme for MemoryScheme {
    fn scheme(&self) -> &str {
        "memory"
    }

    fn open(&mut self, _: &Url, _: usize) -> Result<Box<Resource>> {
        let string = format!("Memory Used: {} KB\nMemory Free: {} KB",
                             memory::memory_used() / 1024,
                             memory::memory_free() / 1024);
        Ok(box VecResource::new(Url::from_str("memory:"), string.into_bytes()))
    }
}
