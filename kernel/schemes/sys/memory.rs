use alloc::boxed::Box;

use arch::memory;

use collections::string::ToString;

use fs::{Resource, VecResource};

use system::error::Result;
use system::syscall::MODE_FILE;

pub fn resource() -> Result<Box<Resource>> {
    let string = format!("Memory Used: {} KB\nMemory Free: {} KB\n",
                         memory::memory_used() / 1024,
                         memory::memory_free() / 1024);
    Ok(box VecResource::new("sys:/memory".to_string(), string.into_bytes(), MODE_FILE))
}
