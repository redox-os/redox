use redox::*;

// nvp implementation version
const NV_VERSION: i32 = 0;

// nvlist header
pub struct NvList {
    pub version: i32,
    pub nvflag:  u32, // persistent flags
    pub pairs: Vec<(String, NvValue)>,
}

impl NvList {
    pub fn new(nvflag: u32) -> Self {
        NvList {
            version: NV_VERSION,
            nvflag: nvflag,
            pairs: Vec::new(),
        }
    }
}

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
    Uint8Array,
}

pub enum NvValue {
    Unknown,
    Boolean,
    Byte(u8),
    Int16(i16),
    Uint16(u16),
    Int32(i32),
    Uint32(u32),
    Int64(i64),
    Uint64(u64),
    String(String),
    ByteArray(Vec<u8>),
    Int16Array(Vec<i16>),
    Uint16Array(Vec<u16>),
    Int32Array(Vec<i32>),
    Uint32Array(Vec<u32>),
    Int64Array(Vec<i64>),
    Uint64Array(Vec<u64>),
    StringArray(Vec<String>),
    HrTime,
    NvList(Box<NvList>),
    NvListArray(Vec<Box<NvList>>),
    BooleanValue(bool),
    Int8(i8),
    Uint8(u8),
    BooleanArray(Vec<bool>),
    Int8Array(Vec<i8>),
    Uint8Array(Vec<u8>),
}
