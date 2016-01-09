use std::{mem, ptr};

use super::from_bytes::FromBytes;
use super::block_ptr::BlockPtr;

const UBERBLOCK_MAGIC: u64 = 0x00bab10c; // oo-ba-bloc!
pub const UBERBLOCK_SHIFT: u64 = 10;         // up to 1K

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct Uberblock {
    pub magic: u64,
    pub version: u64,
    pub txg: u64,
    pub guid_sum: u64,
    pub timestamp: u64,
    pub rootbp: BlockPtr,
}

impl Uberblock {
    pub fn magic_little() -> u64 {
        return 0x0cb1ba00;
    }

    pub fn magic_big() -> u64 {
        return 0x00bab10c;
    }
}

impl FromBytes for Uberblock {
    fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() >= mem::size_of::<Uberblock>() {
            let uberblock = unsafe { ptr::read(data.as_ptr() as *const Uberblock) };
            if uberblock.magic == Uberblock::magic_little() {
                Ok(uberblock)
            } else if uberblock.magic == Uberblock::magic_big() {
                Ok(uberblock)
            } else {
                Err("Error: Invalid uberblock magic number".to_string())
            }
        } else {
            Err(format!("Error: Need {} bytes to read uberblock, only {} in buffer",
                        mem::size_of::<Uberblock>(),
                        data.len()))
        }
    }
}
