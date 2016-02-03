use std::{mem, ptr};

pub trait FromBytes: Sized {
    fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() >= mem::size_of::<Self>() {
            let s = unsafe { ptr::read(data.as_ptr() as *const Self) };
            Ok(s)
        } else {
            Err(format!("Error: bytes length of {} not long enough for the byte size of {}",
                        data.len(),
                        mem::size_of::<Self>()))
        }
    }
}

impl FromBytes for u64 {}
