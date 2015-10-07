use core::mem;

use super::nvpair::{NV_VERSION, NvList, NvValue};
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
pub fn encode_nv_list(xdr: &mut xdr::Xdr, nv_list: &NvList) -> xdr::XdrResult<()> {
    try!(encode_nv_list_header(xdr));

    // Encode version and nvflag
    try!(xdr.encode_i32(nv_list.version));
    try!(xdr.encode_u32(nv_list.nvflag));

    // Encode the pairs
    for &(ref name, ref value) in &nv_list.pairs {
        // Encode encoded/decoded size
        let encoded_size = 0;
        let decoded_size = 0;

        // Encode name
        try!(xdr.encode_string(name));

        // TODO

        // Encode data type
        //try!(xdr.encode_i32(value.get_data_type()));

        // Encode the number of elements
        //try!(xdr.encode_i32(value.num_elements()));

        // Encode the value
    }

    // Encode 2 terminating zeros
    try!(xdr.encode_i32(0));
    try!(xdr.encode_i32(0));
    Ok(())
}

fn encode_nv_list_header(xdr: &mut xdr::Xdr) -> xdr::XdrResult<()> {
    let header =
        NvsHeader {
            encoding: NV_ENCODE_XDR,
            endian: NV_LITTLE_ENDIAN,
            reserved1: 0,
            reserved2: 0,
        };
    let header_bytes: [u8; 4] = unsafe { mem::transmute(header) };
    try!(xdr.encode_bytes(&header_bytes));
    Ok(())
}

pub fn decode_nv_list(xdr: &mut xdr::Xdr) -> xdr::XdrResult<NvList> {
    try!(decode_nv_list_header(xdr));

    // Decode version and nvflag
    let version = try!(xdr.decode_i32());
    let nvflags = try!(xdr.decode_u32());

    // TODO: Give an actual error
    if version != NV_VERSION {
        return Err(xdr::XdrError);
    }

    let mut nv_list = NvList::new(NV_UNIQUE_NAME);

    // Decode the pairs
    loop {
        // Decode decoded/decoded size
        let encoded_size = try!(xdr.decode_u32());
        let decoded_size = try!(xdr.decode_u32());

        // Check for 2 terminating zeros
        if (encoded_size == 0 && decoded_size == 0) {
            break;
        }

        // Decode name
        let name = try!(xdr.decode_string());

        // Decode data type
        let data_type = try!(xdr.decode_i32());

        // Decode the number of elements
        let num_elements = try!(xdr.decode_i32());

        // Decode the value
        let value = NvValue::Uint8(42);
        
        // Add the value to the list
        nv_list.pairs.push((name, value));
    }

    Ok(nv_list)
}

fn decode_nv_list_header(xdr: &mut xdr::Xdr) -> xdr::XdrResult<()> {
    let bytes = try!(xdr.decode_bytes());
    let header: &NvsHeader = unsafe { mem::transmute(&bytes[0]) };
    
    if header.encoding != NV_ENCODE_XDR {
        return Err(xdr::XdrError);
    }
    Ok(())
}
