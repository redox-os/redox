use super::from_bytes::FromBytes;
use super::dvaddr::DVAddr;

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct BlockPtr {
    pub dvas: [DVAddr; 3],
    pub flags_size: u64,
    pub padding: [u64; 3],
    pub birth_txg: u64,
    pub fill_count: u64,
    pub checksum: [u64; 4],
}

impl BlockPtr {
    pub fn level(&self) -> u64 {
        (self.flags_size >> 56) & 0x7F
    }

    pub fn object_type(&self) -> u64 {
        (self.flags_size >> 48) & 0xFF
    }

    pub fn checksum(&self) -> u64 {
        (self.flags_size >> 40) & 0xFF
    }

    pub fn compression(&self) -> u64 {
        (self.flags_size >> 32) & 0xFF
    }

    pub fn lsize(&self) -> u64 {
        (self.flags_size & 0xFFFF) + 1
    }

    pub fn psize(&self) -> u64 {
        ((self.flags_size >> 16) & 0xFFFF) + 1
    }
}

impl FromBytes for BlockPtr {}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct Gang {
    pub bps: [BlockPtr; 3],
    pub padding: [u64; 14],
    pub magic: u64,
    pub checksum: u64,
}

impl Gang {
    pub fn magic() -> u64 {
        return 0x117a0cb17ada1002;
    }
}
