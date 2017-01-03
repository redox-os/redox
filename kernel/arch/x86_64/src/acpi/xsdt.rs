use core::mem;

use super::sdt::Sdt;

#[derive(Debug)]
pub struct Xsdt(&'static Sdt);

impl Xsdt {
    pub fn new(sdt: &'static Sdt) -> Option<Xsdt> {
        if &sdt.signature == b"XSDT" {
            Some(Xsdt(sdt))
        } else {
            None
        }
    }

    pub fn iter(&self) -> XsdtIter {
        XsdtIter {
            sdt: self.0,
            i: 0
        }
    }
}

pub struct XsdtIter {
    sdt: &'static Sdt,
    i: usize
}

impl Iterator for XsdtIter {
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
