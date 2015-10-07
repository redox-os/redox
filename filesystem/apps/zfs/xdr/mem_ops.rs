use core::mem;

use redox::*;

use super::{XdrOps, XdrError, XdrResult};

pub struct MemOps {
    pos: usize,
    buffer: Vec<u8>,
}

impl XdrOps for MemOps {
    fn get_long(&mut self) -> XdrResult<usize> {
        if self.buffer.len()-self.pos < 4 {
            Err(XdrError)
        } else {
            let d: &usize = unsafe { mem::transmute(&self.buffer[self.pos]) };
            self.pos += 4;
            Ok(usize::from_be(*d))
        }
    }

    fn put_long(&mut self, l: usize) -> XdrResult<()> {
        Ok(())
    }

    fn get_i32(&mut self) -> XdrResult<i32> {
        Ok(0)
    }

    fn put_i32(&mut self, i: i32) -> XdrResult<()> {
        Ok(())
    }

    fn get_bytes(&mut self, bytes: &mut [u8]) -> XdrResult<()> {
        Ok(())
    }

    fn put_bytes(&mut self, bytes: &[u8]) -> XdrResult<()> {
        Ok(())
    }

    fn get_pos(&self) -> usize {
        self.pos
    }

    fn set_pos(&mut self, offset: usize) -> XdrResult<()> {
        Ok(())
    }

    fn destroy(&mut self) {
    }
}
