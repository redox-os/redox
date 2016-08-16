use core::mem;

use super::sdt::SDT;

#[derive(Debug)]
pub struct XSDT(&'static SDT);

impl XSDT {
    pub fn new(sdt: &'static SDT) -> Option<XSDT> {
        if &sdt.signature == b"XSDT" {
            Some(XSDT(sdt))
        } else {
            None
        }
    }

    pub fn iter(&self) -> XSDTIter {
        XSDTIter {
            sdt: self.0,
            i: 0
        }
    }
}

pub struct XSDTIter {
    sdt: &'static SDT,
    i: usize
}

impl Iterator for XSDTIter {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.sdt.data_len()/mem::size_of::<u64>() {
            let item = unsafe { *(self.sdt.data_address() as *const u64).offset(self.i as isize) };
            self.i += 1;
            Some(item as usize)
        } else {
            None
        }
    }
}
