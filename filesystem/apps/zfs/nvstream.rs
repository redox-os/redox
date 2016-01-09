use std::mem;

use super::nvpair::{DataType, NV_VERSION, NvList, NvValue};
use super::xdr;

// nvlist pack encoding
const NV_ENCODE_NATIVE: u8 = 0;
const NV_ENCODE_XDR: u8 = 1;

// nvlist pack endian
const NV_BIG_ENDIAN: u8 = 0;
const NV_LITTLE_ENDIAN: u8 = 1;

// nvlist persistent unique name flags, stored in nvl_nvflags
const NV_UNIQUE_NAME: u32 = 0x1;
const NV_UNIQUE_NAME_TYPE: u32 = 0x2;

// nvlist lookup pairs related flags
const NV_FLAG_NOENTOK: isize = 0x1;

// NvList XDR format:
// - header (encoding and endian): 4 bytes
// - nvl version: 4 bytes
// - nv flags: 4 bytes
// - nv pairs:
//   - encoded size: 4 bytes
//   - decoded size: 4 bytes
//   - name: xdr string | len: 4 bytes, data: len+(4 - len%4) bytes
//   - data type: 4 bytes
//   - num elements: 4 bytes
//   - data
// - 2 terminating zeros: 4 bytes
//
// NOTE: XDR aligns all of the smaller integer types to be 4 bytes, so `encode_u8` is actually
// writing 4 bytes
//
// I don't know why the ZFS developers decided to use i32's everywhere. Even for clearly
// unsigned things like array lengths.

/// Name value stream header
#[derive(Debug)]
pub struct NvsHeader {
    encoding: u8, // nvs encoding method
    endian: u8, // nvs endian
    reserved1: u8, // reserved for future use
    reserved2: u8, // reserved for future use
}

/// Encodes a NvList in XDR format
pub fn encode_nv_list(xdr: &mut xdr::Xdr, nv_list: &NvList) -> xdr::XdrResult<()> {
    try!(encode_nv_list_header(xdr));

    // Encode version and nvflag
    try!(xdr.encode_i32(nv_list.version));
    try!(xdr.encode_u32(nv_list.nvflag));

    // Encode the pairs
    for &(ref name, ref value) in &nv_list.pairs {
        // Encode name
        // let encoded_size = 0;
        // let decoded_size = 0;
        try!(xdr.encode_string(name));

        // TODO

        // Encode data type
        try!(xdr.encode_u8(value.data_type().to_u8()));

        // Encode the number of elements
        try!(xdr.encode_i32(value.num_elements() as i32));

        // Encode the value
    }

    // Encode 2 terminating zeros
    try!(xdr.encode_i32(0));
    try!(xdr.encode_i32(0));
    Ok(())
}

fn encode_nv_list_header(xdr: &mut xdr::Xdr) -> xdr::XdrResult<()> {
    let header = NvsHeader {
        encoding: NV_ENCODE_XDR,
        endian: NV_LITTLE_ENDIAN,
        reserved1: 0,
        reserved2: 0,
    };
    let header_bytes: [u8; 4] = unsafe { mem::transmute(header) };
    try!(xdr.encode_opaque(&header_bytes));
    Ok(())
}

/// Decodes a NvList in XDR format
pub fn decode_nv_list(xdr: &mut xdr::Xdr) -> xdr::XdrResult<NvList> {
    try!(decode_nv_list_header(xdr));

    decode_nv_list_embedded(xdr)
}

pub fn decode_nv_list_embedded(xdr: &mut xdr::Xdr) -> xdr::XdrResult<NvList> {
    // Decode version and nvflag
    let version = try!(xdr.decode_i32());
    let nvflag = try!(xdr.decode_u32());

    // TODO: Give an actual error
    if version != NV_VERSION {
        return Err(xdr::XdrError);
    }

    let mut nv_list = NvList::new(nvflag);

    // Decode the pairs
    loop {
        // Decode decoded/decoded size
        let encoded_size = try!(xdr.decode_u32());
        let decoded_size = try!(xdr.decode_u32());

        // Check for 2 terminating zeros
        if encoded_size == 0 && decoded_size == 0 {
            break;
        }

        // Decode name
        let name = try!(xdr.decode_string());

        // Decode data type
        let data_type = match DataType::from_u8(try!(xdr.decode_u8())) {
            Some(dt) => dt,
            None => {
                return Err(xdr::XdrError);
            }
        };

        // Decode the number of elements
        let num_elements = try!(xdr.decode_i32()) as usize;

        // Decode the value
        let value = try!(decode_nv_value(xdr, data_type, num_elements));

        // Add the value to the list
        nv_list.pairs.push((name, value));
    }

    Ok(nv_list)
}

fn decode_nv_list_header(xdr: &mut xdr::Xdr) -> xdr::XdrResult<()> {
    let mut bytes: [u8; 4] = [0; 4];
    try!(xdr.decode_opaque(&mut bytes));
    let header: NvsHeader = unsafe { mem::transmute(bytes) };

    if header.encoding != NV_ENCODE_XDR {
        return Err(xdr::XdrError);
    }
    Ok(())
}

fn decode_nv_value(xdr: &mut xdr::Xdr,
                   data_type: DataType,
                   num_elements: usize)
                   -> xdr::XdrResult<NvValue> {
    match data_type {
        DataType::Unknown => Ok(NvValue::Unknown),
        DataType::Boolean => Ok(NvValue::Boolean),
        DataType::Byte => Ok(NvValue::Byte(try!(xdr.decode_u8()))),
        DataType::Int16 => Ok(NvValue::Int16(try!(xdr.decode_i16()))),
        DataType::Uint16 => Ok(NvValue::Uint16(try!(xdr.decode_u16()))),
        DataType::Int32 => Ok(NvValue::Int32(try!(xdr.decode_i32()))),
        DataType::Uint32 => Ok(NvValue::Uint32(try!(xdr.decode_u32()))),
        DataType::Int64 => Ok(NvValue::Int64(try!(xdr.decode_i64()))),
        DataType::Uint64 => Ok(NvValue::Uint64(try!(xdr.decode_u64()))),
        DataType::String => Ok(NvValue::String(try!(xdr.decode_string()))),
        DataType::ByteArray => {
            let mut v = vec![0; num_elements];
            for v in &mut v {
                *v = try!(xdr.decode_u8());
            }
            Ok(NvValue::ByteArray(v))
        }
        DataType::Int16Array => {
            let mut v = vec![0; num_elements];
            for v in &mut v {
                *v = try!(xdr.decode_i16());
            }
            Ok(NvValue::Int16Array(v))
        }
        DataType::Uint16Array => {
            let mut v = vec![0; num_elements];
            for v in &mut v {
                *v = try!(xdr.decode_u16());
            }
            Ok(NvValue::Uint16Array(v))
        }
        DataType::Int32Array => {
            let mut v = vec![0; num_elements];
            for v in &mut v {
                *v = try!(xdr.decode_i32());
            }
            Ok(NvValue::Int32Array(v))
        }
        DataType::Uint32Array => {
            let mut v = vec![0; num_elements];
            for v in &mut v {
                *v = try!(xdr.decode_u32());
            }
            Ok(NvValue::Uint32Array(v))
        }
        DataType::Int64Array => {
            let mut v = vec![0; num_elements];
            for v in &mut v {
                *v = try!(xdr.decode_i64());
            }
            Ok(NvValue::Int64Array(v))
        }
        DataType::Uint64Array => {
            let mut v = vec![0; num_elements];
            for v in &mut v {
                *v = try!(xdr.decode_u64());
            }
            Ok(NvValue::Uint64Array(v))
        }
        DataType::StringArray => {
            let mut v = vec![0; num_elements];
            for v in &mut v {
                *v = try!(xdr.decode_u64());
            }
            Ok(NvValue::Uint64Array(v))
        }
        DataType::HrTime => Ok(NvValue::HrTime(try!(xdr.decode_i64()))),
        DataType::NvList => {
            let nv_list = try!(decode_nv_list_embedded(xdr));
            Ok(NvValue::NvList(nv_list))
        }
        DataType::NvListArray => {
            let mut v = Vec::with_capacity(num_elements);
            for _ in 0..num_elements {
                v.push(try!(decode_nv_list_embedded(xdr)));
            }
            Ok(NvValue::NvListArray(v))
        }
        DataType::BooleanValue => Ok(NvValue::BooleanValue(try!(xdr.decode_bool()))),
        DataType::Int8 => Ok(NvValue::Int8(try!(xdr.decode_i8()))),
        DataType::Uint8 => Ok(NvValue::Uint8(try!(xdr.decode_u8()))),
        DataType::BooleanArray => {
            let mut v = vec![false; num_elements];
            for v in &mut v {
                *v = try!(xdr.decode_bool());
            }
            Ok(NvValue::BooleanArray(v))
        }
        DataType::Int8Array => {
            let mut v = vec![0; num_elements];
            for v in &mut v {
                *v = try!(xdr.decode_i8());
            }
            Ok(NvValue::Int8Array(v))
        }
        DataType::Uint8Array => {
            let mut v = vec![0; num_elements];
            for v in &mut v {
                *v = try!(xdr.decode_u8());
            }
            Ok(NvValue::Uint8Array(v))
        }
    }
}
