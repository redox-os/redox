use alloc::boxed::Box;

use collections::string::ToString;

use fs::{Resource, VecResource};

use system::error::Result;
use system::syscall::MODE_FILE;

pub fn resource() -> Result<Box<Resource>> {
    let mut string = format!("{:<6}{:<10}{}\n", "PATH", "SIZE", "NAME");

    for (i, disk) in unsafe { &mut *::env().disks.get() }.iter().enumerate() {
        let size = unsafe { & *disk.get() }.size();
        let size_string = if size >= 1024 * 1024 * 1024 {
            format!("{} GB", size / 1024 / 1024 / 1024)
        } else if size >= 1024 * 1024 {
            format!("{} MB", size / 1024 / 1024)
        } else if size >= 1024 {
            format!("{} KB", size / 1024)
        } else {
            format!("{} B", size)
        };
        string.push_str(&format!("{:<6}{:<10}{}\n", i, size_string, unsafe { & *disk.get() }.name()));
    }

    Ok(box VecResource::new("sys:/disk".to_string(), string.into_bytes(), MODE_FILE))
}
