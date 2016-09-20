use core::ops::{Deref, DerefMut};
use core::{mem, slice};

#[derive(Copy, Clone, Debug, Default)]
#[repr(packed)]
pub struct Packet {
    pub id: usize,
    pub a: usize,
    pub b: usize,
    pub c: usize,
    pub d: usize
}

impl Deref for Packet {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self as *const Packet as *const u8, mem::size_of::<Packet>()) as &[u8]
        }
    }
}

impl DerefMut for Packet {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self as *mut Packet as *mut u8, mem::size_of::<Packet>()) as &mut [u8]
        }
    }
}
