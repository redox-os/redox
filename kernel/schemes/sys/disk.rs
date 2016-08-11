use alloc::boxed::Box;

use collections::string::{String, ToString};

use fs::{Resource, VecResource};

use system::error::Result;
use system::syscall::MODE_FILE;

pub fn resource() -> Result<Box<Resource>> {
    let mut string = String::new();

    for (i, disk) in unsafe { &mut *::env().disks.get() }.iter().enumerate() {
        string.push_str(&format!("{:<6}{}\n", i, unsafe { & *disk.get() }.name()));
    }

    Ok(box VecResource::new("sys:/disk".to_string(), string.into_bytes(), MODE_FILE))
}
