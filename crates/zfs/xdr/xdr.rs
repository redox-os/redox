// use std::*;

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
    /// Get a i64 from underlying stream
    fn get_i64(&mut self) -> XdrResult<i64>;

    /// Put a i64 to underlying stream
    fn put_i64(&mut self, l: i64) -> XdrResult<()>;

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
// fn inline(&mut self, len: usize) -> *mut i32;

// TODO: Not sure if we'll need this?
// Change, retrieve client info
// fn control(&mut self, req: isize, op: void *);
}

pub trait Xdr {
    fn encode_bool(&mut self, i: bool) -> XdrResult<()>;
    fn decode_bool(&mut self) -> XdrResult<bool>;

    fn encode_i8(&mut self, i: i8) -> XdrResult<()>;
    fn decode_i8(&mut self) -> XdrResult<i8>;

    fn encode_u8(&mut self, u: u8) -> XdrResult<()>;
    fn decode_u8(&mut self) -> XdrResult<u8>;

    fn encode_i16(&mut self, i: i16) -> XdrResult<()>;
    fn decode_i16(&mut self) -> XdrResult<i16>;

    fn encode_u16(&mut self, u: u16) -> XdrResult<()>;
    fn decode_u16(&mut self) -> XdrResult<u16>;

    fn encode_i32(&mut self, i: i32) -> XdrResult<()>;
    fn decode_i32(&mut self) -> XdrResult<i32>;

    fn encode_u32(&mut self, u: u32) -> XdrResult<()>;
    fn decode_u32(&mut self) -> XdrResult<u32>;

    fn encode_i64(&mut self, i: i64) -> XdrResult<()>;
    fn decode_i64(&mut self) -> XdrResult<i64>;

    fn encode_u64(&mut self, u: u64) -> XdrResult<()>;
    fn decode_u64(&mut self) -> XdrResult<u64>;

    fn encode_opaque(&mut self, bytes: &[u8]) -> XdrResult<()>;
    fn decode_opaque(&mut self, bytes: &mut [u8]) -> XdrResult<()>;

    fn encode_bytes(&mut self, bytes: &[u8]) -> XdrResult<()>;
    fn decode_bytes(&mut self) -> XdrResult<Vec<u8>>;

    fn encode_string(&mut self, string: &String) -> XdrResult<()>;
    fn decode_string(&mut self) -> XdrResult<String>;
}

impl<T: XdrOps> Xdr for T {
    fn encode_bool(&mut self, b: bool) -> XdrResult<()> {
        let i = match b {
            false => 0,
            true => 1,
        };
        self.put_i32(i)
    }

    fn decode_bool(&mut self) -> XdrResult<bool> {
        let i = try!(self.get_i32());
        match i {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(XdrError),
        }
    }

    fn encode_i8(&mut self, i: i8) -> XdrResult<()> {
        self.put_i32(i as i32)
    }

    fn decode_i8(&mut self) -> XdrResult<i8> {
        self.get_i32().map(|x| x as i8)
    }

    fn encode_u8(&mut self, u: u8) -> XdrResult<()> {
        self.put_i32(u as i32)
    }

    fn decode_u8(&mut self) -> XdrResult<u8> {
        self.get_i32().map(|x| x as u8)
    }

    fn encode_i16(&mut self, i: i16) -> XdrResult<()> {
        self.put_i32(i as i32)
    }

    fn decode_i16(&mut self) -> XdrResult<i16> {
        self.get_i32().map(|x| x as i16)
    }

    fn encode_u16(&mut self, u: u16) -> XdrResult<()> {
        self.put_i32(u as i32)
    }

    fn decode_u16(&mut self) -> XdrResult<u16> {
        self.get_i32().map(|x| x as u16)
    }

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

    fn encode_i64(&mut self, i: i64) -> XdrResult<()> {
        self.put_i64(i)
    }

    fn decode_i64(&mut self) -> XdrResult<i64> {
        self.get_i64()
    }

    fn encode_u64(&mut self, u: u64) -> XdrResult<()> {
        self.put_i64(u as i64)
    }

    fn decode_u64(&mut self) -> XdrResult<u64> {
        self.get_i64().map(|x| x as u64)
    }

    fn encode_opaque(&mut self, bytes: &[u8]) -> XdrResult<()> {
        // XDR byte strings always have len%4 == 0
        let crud: [u8; 4] = [0; 4];
        let mut round_up = bytes.len() % 4;
        if round_up > 0 {
            round_up = 4 - round_up;
        }
        try!(self.put_bytes(bytes));
        try!(self.put_bytes(&crud[0..round_up]));
        Ok(())
    }

    fn decode_opaque(&mut self, bytes: &mut [u8]) -> XdrResult<()> {
        // XDR byte strings always have len%4 == 0
        let mut crud: [u8; 4] = [0; 4];
        let mut round_up = bytes.len() % 4;
        if round_up > 0 {
            round_up = 4 - round_up;
        }
        try!(self.get_bytes(bytes));
        try!(self.get_bytes(&mut crud[0..round_up]));
        Ok(())
    }

    fn encode_bytes(&mut self, bytes: &[u8]) -> XdrResult<()> {
        try!(self.encode_u32(bytes.len() as u32));
        self.encode_opaque(bytes)
    }

    fn decode_bytes(&mut self) -> XdrResult<Vec<u8>> {
        let count = try!(self.decode_u32());
        let mut bytes = vec![0; count as usize];
        try!(self.decode_opaque(&mut bytes[..]));
        Ok(bytes)
    }

    fn encode_string(&mut self, string: &String) -> XdrResult<()> {
        try!(self.encode_u32(string.as_bytes().len() as u32));
        self.encode_opaque(string.as_bytes())
    }

    fn decode_string(&mut self) -> XdrResult<String> {
        let count = try!(self.decode_u32());
        if count > 1024 {
            return Err(XdrError);
        }
        let mut bytes = vec![0; count as usize];
        try!(self.decode_opaque(&mut bytes[..]));
        String::from_utf8(bytes).map_err(|_| XdrError)
    }
}
