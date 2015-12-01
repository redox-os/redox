use std::fmt;

#[derive(Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(packed)]
pub struct DVAddr {
    pub vdev: u64,
    pub offset: u64,
}

impl DVAddr {
    /// Sector address is the offset plus two vdev labels and one boot block (4 MB, or 8192 sectors)
    pub fn sector(&self) -> u64 {
        self.offset() + 0x2000
    }

    pub fn gang(&self) -> bool {
        if self.offset & 0x8000000000000000 == 1 {
            true
        } else {
            false
        }
    }

    pub fn offset(&self) -> u64 {
        self.offset & 0x7FFFFFFFFFFFFFFF
    }

    pub fn asize(&self) -> u64 {
        (self.vdev & 0xFFFFFF) + 1
    }
}

impl fmt::Debug for DVAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f,
                    "DVAddr {{ offset: {:X}, gang: {}, asize: {:X} }}\n",
                    self.offset(),
                    self.gang(),
                    self.asize()));
        Ok(())
    }
}
