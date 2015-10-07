use redox::*;

#[derive(Debug)]
pub struct XdrError;

pub type XdrResult<T> = Result<T, XdrError>;

pub enum XdrOp {
    Encode,
    Decode,
    Free,
}

// TODO: Return `XdrResult` instead
pub trait XdrOps {
    /// Get a usize from underlying stream
    fn get_usize(&mut self) -> XdrResult<usize>;

    /// Put a usize to underlying stream
    fn put_usize(&mut self, l: usize) -> XdrResult<()>;

    /// Get a i32 from underlying stream
    fn get_i32(&mut self) -> XdrResult<i32>;

    /// Put a i32 to underlying stream
    fn put_i32(&mut self, i: i32) -> XdrResult<()>;

    /// Get some bytes from the underlying stream
    fn get_bytes(&mut self, bytes: &mut [u8]) -> XdrResult<()>;

    /// Put some bytes into the underlying stream
    fn put_bytes(&mut self, bytes: &[u8]) -> XdrResult<()>;

    /// Returns bytes off from beginning
    fn get_pos(&self) -> usize;

    /// Lets you reposition the stream
    fn set_pos(&mut self, offset: usize) -> XdrResult<()>;

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
    fn encode_i32(&mut self, i: i32) -> XdrResult<()>;
    fn decode_i32(&mut self) -> XdrResult<i32>;

    fn encode_u32(&mut self, u: u32) -> XdrResult<()>;
    fn decode_u32(&mut self) -> XdrResult<u32>;

    fn encode_opaque(&mut self, bytes: &[u8]) -> XdrResult<()>;
    fn decode_opaque(&mut self, bytes: &mut [u8]) -> XdrResult<()>;

    fn encode_bytes(&mut self, bytes: &[u8]) -> XdrResult<()>;
    fn decode_bytes(&mut self) -> XdrResult<Vec<u8>>;

    fn encode_string(&mut self, string: &String) -> XdrResult<()>;
    fn decode_string(&mut self) -> XdrResult<String>;
}

impl<T: XdrOps> Xdr for T {
    fn encode_i32(&mut self, i: i32) -> XdrResult<()> {
        self.put_i32(i)
    }

    fn decode_i32(&mut self) -> XdrResult<i32> {
        self.get_i32()
    }

    fn encode_u32(&mut self, u: u32) -> XdrResult<()> {
        self.put_i32(u as i32)
    }

    fn decode_u32(&mut self) -> XdrResult<u32> {
        self.get_i32().map(|x| x as u32)
    }

    fn encode_opaque(&mut self, bytes: &[u8]) -> XdrResult<()> {
        // XDR byte strings always have len%4 == 0
        let mut crud: [u8; 4] = [0; 4];
        let round_up = 4 - (bytes.len()%4);
        try!(self.put_bytes(bytes));
        try!(self.put_bytes(&crud[0..round_up]));
        Ok(())
    }

    fn decode_opaque(&mut self, bytes: &mut [u8]) -> XdrResult<()> {
        // XDR byte strings always have len%4 == 0
        let mut crud: [u8; 4] = [0; 4];
        let round_up = 4 - (bytes.len()%4);
        try!(self.get_bytes(bytes));
        try!(self.get_bytes(&mut crud[0..round_up]));
        Ok(())
    }

    fn encode_bytes(&mut self, bytes: &[u8]) -> XdrResult<()> {
        self.encode_opaque(bytes)
    }

    fn decode_bytes(&mut self) -> XdrResult<Vec<u8>> {
        let count = try!(self.decode_u32());
        let mut bytes = vec![0; count as usize];
        try!(self.decode_opaque(&mut bytes[..]));
        Ok(bytes)
    }

    fn encode_string(&mut self, string: &String) -> XdrResult<()> {
        self.encode_opaque(string.as_bytes())
    }

    fn decode_string(&mut self) -> XdrResult<String> {
        let count = try!(self.decode_u32());
        let mut bytes = vec![0; count as usize];
        try!(self.decode_opaque(&mut bytes[..]));
        String::from_utf8(bytes).map_err(|_| XdrError)
    }
}
