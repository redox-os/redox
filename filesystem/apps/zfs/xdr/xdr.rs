use redox::*;

pub enum XdrOp {
    Encode,
    Decode,
    Free,
}

// TODO: Return `XdrResult` instead
pub trait XdrOps {
    /// Get a long from underlying stream
    fn get_long(&mut self) -> usize;

    /// Put a long to underlying stream
    fn put_long(&mut self, l: usize) -> bool;

    /// Get a i32 from underlying stream
    fn get_i32(&mut self) -> i32;

    /// Put a i32 to underlying stream
    fn put_i32(&mut self, i: i32) -> bool;

    /// Get some bytes from the underlying stream
    fn get_bytes(&mut self, bytes: &mut [u8]) -> bool;

    /// Put some bytes into the underlying stream
    fn put_bytes(&mut self, bytes: &[u8]) -> bool;

    /// Returns bytes off from beginning
    fn get_pos(&self) -> usize;

    /// Lets you reposition the stream
    fn set_pos(&mut self, offset: usize) -> bool;

    // TODO: Not sure if we'll need this?
    // Buf quick ptr to buffered data
    //fn inline(&mut self, len: usize) -> *mut i32;

    /// Free privates of this xdr_stream
    fn destroy(&mut self);

    // TODO: Not sure if we'll need this?
    // Change, retrieve client info
    //fn control(&mut self, req: isize, op: void *);
}

pub trait Xdr {
    fn encode_i32(&mut self, i: i32) -> bool;
    fn decode_i32(&mut self) -> i32;

    fn encode_u32(&mut self, u: u32) -> bool;
    fn decode_u32(&mut self) -> u32;

    fn encode_opaque(&mut self, bytes: &[u8]) -> bool;
    fn decode_opaque(&mut self, bytes: &mut [u8]) -> bool;

    fn encode_bytes(&mut self, bytes: &[u8]) -> bool;
    fn decode_bytes(&mut self) -> Vec<u8>;

    fn encode_string(&mut self, string: &String) -> bool;
    fn decode_string(&mut self) -> String;
}

impl<T: XdrOps> Xdr for T {
    fn encode_i32(&mut self, i: i32) -> bool {
        self.put_i32(i)
    }

    fn decode_i32(&mut self) -> i32 {
        self.get_i32()
    }

    fn encode_u32(&mut self, u: u32) -> bool {
        self.put_i32(u as i32)
    }

    fn decode_u32(&mut self) -> u32 {
        self.get_i32() as u32
    }

    fn encode_opaque(&mut self, bytes: &[u8]) -> bool {
        // XDR byte strings always have len%4 == 0
        let mut crud: [u8; 4] = [0; 4];
        let round_up = 4 - (bytes.len()%4);
        self.put_bytes(bytes);
        self.put_bytes(&crud[0..round_up]);
        true
    }

    fn decode_opaque(&mut self, bytes: &mut [u8]) -> bool {
        // XDR byte strings always have len%4 == 0
        let mut crud: [u8; 4] = [0; 4];
        let round_up = 4 - (bytes.len()%4);
        self.get_bytes(bytes);
        self.get_bytes(&mut crud[0..round_up]);
        true
    }

    fn encode_bytes(&mut self, bytes: &[u8]) -> bool {
        self.encode_opaque(bytes)
    }

    fn decode_bytes(&mut self) -> Vec<u8> {
        let count = self.decode_u32();
        let mut bytes = vec![0; count as usize];
        self.decode_opaque(&mut bytes[..]);
        bytes
    }

    fn encode_string(&mut self, string: &String) -> bool {
        // TODO
        true
    }

    fn decode_string(&mut self) -> String {
        // TODO
        String::new()
    }
}
