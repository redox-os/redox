use alloc::boxed::Box;

use arch::memory;

use collections::string::ToString;

use fs::{KScheme, Resource, VecResource};

use system::error::Result;

/// A memory scheme
pub struct MemoryScheme;

impl KScheme for MemoryScheme {
    fn scheme(&self) -> &str {
        "memory"
    }

    fn open(&mut self, _: &str, _: usize) -> Result<Box<Resource>> {
        let string = format!("Memory Used: {} KB\nMemory Free: {} KB\n",
                             memory::memory_used() / 1024,
                             memory::memory_free() / 1024);
        Ok(box VecResource::new("memory:".to_string(), string.into_bytes()))
    }
}
