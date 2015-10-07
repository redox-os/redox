use core::mem;

use redox::*;

use super::{XdrOps, XdrError, XdrResult};

pub struct MemOps {
    pos: usize,
    buffer: Vec<u8>,
}

impl MemOps {
    pub fn new(buffer: Vec<u8>) -> MemOps {
        MemOps {
            pos: 0,
            buffer: buffer,
        }
    }
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
        if self.buffer.len()-self.pos < 4 {
            Err(XdrError)
        } else {
            let d: &i32 = unsafe { mem::transmute(&self.buffer[self.pos]) };
            self.pos += 4;
            Ok(i32::from_be(*d))
        }
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

    fn set_pos(&mut self, new_pos: usize) -> XdrResult<()> {
        self.pos = new_pos;
        Ok(())
    }

    fn destroy(&mut self) {
    }
}

#[test]
fn test_mem_ops_u32() {
    let mem_ops = MemOps::new(vec![1, 1, 0, 0]);
    assert!(mem_ops.get_u32() == 257);
}
