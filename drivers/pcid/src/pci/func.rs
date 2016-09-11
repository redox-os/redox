use std::ops::DerefMut;

use super::{PciDev, PciHeader};

pub struct PciFunc<'pci> {
    pub dev: &'pci PciDev<'pci>,
    pub num: u8
}

impl<'pci> PciFunc<'pci> {
    pub fn header(&self) -> Option<PciHeader> {
        if unsafe { self.read(0) } != 0xFFFFFFFF {
            let mut header = PciHeader::default();
            {
                let dwords = header.deref_mut();
                dwords.iter_mut().fold(0usize, |offset, dword| {
                    *dword = unsafe { self.read(offset as u8) };
                    offset + 4
                });
            }
            Some(header)
        } else {
            None
        }
    }

    pub unsafe fn read(&self, offset: u8) -> u32 {
        self.dev.read(self.num, offset)
    }
}
