use std::{mem, ptr};

use super::{XdrOps, XdrError, XdrResult};

pub struct MemOps<'a> {
    pos: usize,
    buffer: &'a mut [u8],
}

impl<'a> MemOps<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        MemOps {
            pos: 0,
            buffer: buffer,
        }
    }
}

// Xdr encodes things in big endian and values are aligned at 4 bytes. For example, a u8 would take
// up 4 bytes when serialized.
impl<'a> XdrOps for MemOps<'a> {
    fn get_i64(&mut self) -> XdrResult<i64> {
        if self.pos >= self.buffer.len() {
            Err(XdrError)
        } else if self.buffer.len() - self.pos < 8 {
            Err(XdrError)
        } else {
            let d: &i64 = unsafe { mem::transmute(&self.buffer[self.pos]) };
            // let val_d = i64::from_be(*d);
            self.pos += 8;
            Ok(i64::from_be(*d))
        }
    }

    fn put_i64(&mut self, l: i64) -> XdrResult<()> {
        if self.pos >= self.buffer.len() || self.buffer.len() - self.pos < 8 {
            // Buffer is too small
            return Err(XdrError);
        }

        let d: &mut i64 = unsafe { mem::transmute(&mut self.buffer[self.pos]) };
        *d = l.to_be();
        self.pos += 8;
        Ok(())
    }

    fn get_i32(&mut self) -> XdrResult<i32> {
        if self.pos >= self.buffer.len() {
            Err(XdrError)
        } else if self.buffer.len() - self.pos < 4 {
            Err(XdrError)
        } else {
            let d: &i32 = unsafe { mem::transmute(&self.buffer[self.pos]) };
            self.pos += 4;
            Ok(i32::from_be(*d))
        }
    }

    fn put_i32(&mut self, i: i32) -> XdrResult<()> {
        if self.pos >= self.buffer.len() || self.buffer.len() - self.pos < 4 {
            // Buffer is too small
            return Err(XdrError);
        }

        let d: &mut i32 = unsafe { mem::transmute(&mut self.buffer[self.pos]) };
        *d = i.to_be();
        self.pos += 4;
        Ok(())
    }

    fn get_bytes(&mut self, bytes: &mut [u8]) -> XdrResult<()> {
        if bytes.is_empty() {
            return Ok(());
        }
        if self.pos >= self.buffer.len() {
            Err(XdrError)
        } else if self.buffer.len() - self.pos < bytes.len() {
            Err(XdrError)
        } else {
            // Technically the upper bound on this slice doesn't have to be there
            let src = self.buffer[self.pos..self.pos + bytes.len()].as_ptr();
            let dst = bytes.as_mut_ptr();
            unsafe {
                ptr::copy(src, dst, bytes.len());
            }
            self.pos += bytes.len();

            Ok(())
        }
    }

    fn put_bytes(&mut self, bytes: &[u8]) -> XdrResult<()> {
        if self.pos >= self.buffer.len() || self.buffer.len() - self.pos < bytes.len() {
            // Buffer is too small
            return Err(XdrError);
        }

        let src = bytes.as_ptr();
        // Technically the upper bound on this slice doesn't have to be there
        let dst = self.buffer[self.pos..self.pos + bytes.len()].as_mut_ptr();
        unsafe {
            ptr::copy(src, dst, bytes.len());
        }
        self.pos += bytes.len();

        Ok(())
    }

    fn get_pos(&self) -> usize {
        self.pos
    }

    fn set_pos(&mut self, new_pos: usize) -> XdrResult<()> {
        self.pos = new_pos;
        Ok(())
    }
}

#[test]
fn test_mem_ops_i64() {
    let mem_ops = MemOps::new(&mut [1, 1, 0, 0]);
    assert!(mem_ops.get_i32() == 257);
}

#[test]
fn test_mem_ops_i64_and_back() {
    let mut mem_ops = MemOps::new(&mut [0; 8]);
    mem_ops.put_i64(424242);
    mem_ops.set_pos(0);
    assert!(mem_ops.get_i64() == 424242);
}

#[test]
fn test_mem_ops_i32() {
    let mem_ops = MemOps::new(&mut [1, 1, 0, 0]);
    assert!(mem_ops.get_i32() == 257);
}

#[test]
fn test_mem_ops_i32_and_back() {
    let mut mem_ops = MemOps::new(&mut [0; 4]);
    mem_ops.put_i32(424242);
    mem_ops.set_pos(0);
    assert!(mem_ops.get_i32() == 424242);
}
