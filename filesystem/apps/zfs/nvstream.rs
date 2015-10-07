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

enum DataType {
    Unknown = 0,
    Boolean,
    Byte,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Int64,
    Uint64,
    String,
    ByteArray,
    Int16Array,
    Uint16Array,
    Int32Array,
    Uint32Array,
    Int64Array,
    Uint64Array,
    StringArray,
    HrTime,
    NvList,
    NvListArray,
    BooleanValue,
    Int8,
    Uint8,
    BooleanArray,
    Int8Array,
    Uint8Array
}

// Name value stream header
pub struct NvsHeader {
    encoding: u8,  // nvs encoding method
    endian: u8,    // nvs endian
    reserved1: u8, // reserved for future use
    reserved2: u8, // reserved for future use
}

pub struct XdrNvListEncoder<T: xdr::XdrOps> {
    xdr_ops: T,
}

impl<T: xdr::XdrOps> XdrNvListEncoder<T> {
    pub fn new(xdr_ops: T) -> XdrNvListEncoder<T> {
        XdrNvListEncoder {
            xdr_ops: xdr_ops,
        }
    }

    pub fn encode(&mut self, nv_list: &NvList) {
        self.encode_header();
    }

    fn encode_header(&mut self) {
        let header =
            NvsHeader {
                encoding: NV_ENCODE_XDR,
                endian: NV_LITTLE_ENDIAN,
                reserved1: 0,
                reserved2: 0,
            };
        let header_bytes: [u8; 4] = unsafe { mem::transmute(header) };
        self.xdr_ops.put_bytes(&header_bytes);
    }
}
