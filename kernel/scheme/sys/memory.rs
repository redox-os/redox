use collections::Vec;

use arch::memory::{free_frames, used_frames};
use syscall::error::Result;

pub fn resource() -> Result<Vec<u8>> {
    let string = format!("Memory Used: {} KB\nMemory Free: {} KB\n", used_frames() * 4, free_frames() * 4);

    Ok(string.into_bytes())
}
