use redox::*;

use super::XdrOps;

pub struct MemOps {
    buffer: Vec<u8>,
}

impl XdrOps for MemOps {
    fn get_long(&mut self) -> usize {
        0
    }

    fn put_long(&mut self, l: usize) -> bool {
        false
    }

    fn get_i32(&mut self) -> i32 {
        0
    }

    fn put_i32(&mut self, i: i32) -> bool {
        false
    }

    fn get_bytes(&mut self, bytes: &mut [u8]) -> bool {
        false
    }

    fn put_bytes(&mut self, bytes: &[u8]) -> bool {
        false
    }

    fn get_pos(&self) -> usize {
        0
    }

    fn set_pos(&mut self, offset: usize) -> bool {
        false
    }

    fn destroy(&mut self) {
    }
}
