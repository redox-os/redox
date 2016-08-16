use core::mem;

use super::sdt::SDT;

#[derive(Debug)]
pub struct RSDT(&'static SDT);

impl RSDT {
    pub fn new(sdt: &'static SDT) -> Option<RSDT> {
        if &sdt.signature == b"RSDT" {
            Some(RSDT(sdt))
        } else {
            None
        }
    }

    pub fn iter(&self) -> RSDTIter {
        RSDTIter {
            sdt: self.0,
            i: 0
        }
    }
}

pub struct RSDTIter {
    sdt: &'static SDT,
    i: usize
}

impl Iterator for RSDTIter {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.sdt.data_len()/mem::size_of::<u32>() {
            let item = unsafe { *(self.sdt.data_address() as *const u32).offset(self.i as isize) };
            self.i += 1;
            Some(item as usize)
        } else {
            None
        }
    }
}
