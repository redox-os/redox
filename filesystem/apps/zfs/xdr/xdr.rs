pub enum XdrOp {
    Encode,
    Decode,
    Free,
}

pub trait XdrOps {
    /// Get a long from underlying stream
    fn get_long(&mut self, l: &mut usize) -> bool;

    /// Put a long to underlying stream
    fn put_long(&mut self, l: usize) -> bool;

    /// Get a i32 from underlying stream
    fn get_i32(&mut self, i: &mut i32) -> bool;

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

pub struct Xdr {
    public: usize,  // pointer to users' data
    ops: Box<XdrOps>,
}


impl Xdr {
    pub fn new(ops: Box<XdrOps>) -> Xdr {
        Xdr {
            public: 0,
            ops: ops,
        }
    }

    pub fn encode_i32(&self, i: i32) {
        self.ops.put_i32(i);
    }

    pub fn decode_i32(&self, i: &mut i32) {
        self.ops.get_i32(i);
    }
}
