use core::mem;

use super::nvpair::NvList;
use super::xdr;

// nvlist pack encoding
const NV_ENCODE_NATIVE: u8 = 0;
const NV_ENCODE_XDR:    u8 = 1;

// nvlist pack endian
const NV_BIG_ENDIAN:    u8 = 0;
const NV_LITTLE_ENDIAN: u8 = 1;

// nvlist persistent unique name flags, stored in nvl_nvflags
const NV_UNIQUE_NAME:      u32 = 0x1;
const NV_UNIQUE_NAME_TYPE: u32 = 0x2;

// nvlist lookup pairs related flags
const NV_FLAG_NOENTOK: isize = 0x1;

// Name value stream header
pub struct NvsHeader {
    encoding: u8,  // nvs encoding method
    endian: u8,    // nvs endian
    reserved1: u8, // reserved for future use
    reserved2: u8, // reserved for future use
}

/// NvList XDR format:
/// - header (encoding and endian): 4 bytes
/// - nvl version: 4 bytes
/// - nv flags: 4 bytes 
/// - nv pairs:
///   - encoded size: 4 bytes
///   - decoded size: 4 bytes
///   - name: xdr string | len: 4 bytes, data: len+(4 - len%4) bytes
///   - data type: 4 bytes
///   - num elements: 4 bytes
///   - data
/// - 2 terminating zeros: 4 bytes
pub struct XdrNvListEncoder<T: xdr::Xdr> {
    xdr_ops: T,
}

impl<T: xdr::Xdr> XdrNvListEncoder<T> {
    pub fn new(xdr_ops: T) -> XdrNvListEncoder<T> {
        XdrNvListEncoder {
            xdr_ops: xdr_ops,
        }
    }

    pub fn encode(&mut self, nv_list: &NvList) -> xdr::XdrResult<()> {
        try!(self.encode_header());

        // Encode version and nvflag
        try!(self.xdr_ops.encode_i32(nv_list.version));
        try!(self.xdr_ops.encode_u32(nv_list.nvflag));

        // Encode the pairs
        for &(ref name, ref value) in &nv_list.pairs {
            // Encode encoded/decoded size
            let encoded_size = 0;
            let decoded_size = 0;

            // Encode name
            try!(self.xdr_ops.encode_string(name));

            // TODO

            // Encode data type
            //try!(self.xdr_ops.encode_i32(value.get_data_type()));

            // Encode the number of elements
            //try!(self.xdr_ops.encode_i32(value.num_elements()));

            // Encode the value
        }

        try!(self.encode_end_zeros());
        Ok(())
    }

    fn encode_header(&mut self) -> xdr::XdrResult<()> {
        let header =
            NvsHeader {
                encoding: NV_ENCODE_XDR,
                endian: NV_LITTLE_ENDIAN,
                reserved1: 0,
                reserved2: 0,
            };
        let header_bytes: [u8; 4] = unsafe { mem::transmute(header) };
        try!(self.xdr_ops.encode_bytes(&header_bytes));
        Ok(())
    }

    fn encode_end_zeros(&mut self) -> xdr::XdrResult<()> {
        try!(self.xdr_ops.encode_i32(0));
        try!(self.xdr_ops.encode_i32(0));
        Ok(())
    }
}
