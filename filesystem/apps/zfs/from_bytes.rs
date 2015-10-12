use redox::*;

pub trait FromBytes: Sized {
    fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() >= mem::size_of::<Self>() {
            let s = unsafe { ptr::read(data.as_ptr() as *const Self) };
            Some(s)
        } else {
            Option::None
        }
    }
}
