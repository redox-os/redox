use core::mem::size_of;
use core::ptr;
use core::slice;

#[repr(packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SDTHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oemid: [u8; 6],
    pub oemtableid: [u8; 8],
    pub oemrevision: u32,
    pub creatorid: u32,
    pub creatorrevision: u32,
}

impl SDTHeader {
    pub fn valid(&self, signature: &str) -> bool {
        if self.signature == signature.as_bytes() {
            let ptr = (self as *const Self) as *const u8;
            let sum: u8 = (0..self.length as isize)
                .fold(0, |sum, i| sum + unsafe { ptr::read(ptr.offset(i)) });

            sum == 0
        } else {
            false
        }
    }

    pub fn data<T>(&self) -> &'static [T] {
        let ptr = ((self as *const Self) as usize + size_of::<Self>()) as *const T;
        let len = (self.length as usize - size_of::<Self>()) / size_of::<T>();
        unsafe { slice::from_raw_parts(ptr, len) }
    }
}
