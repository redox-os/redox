use redox::*;

enum NvValue {
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

// nvlist header
struct NvList {
    version: i32,
    nvflag:  u32, // persistent flags
    pairs: HashMap<String, NvValue>,
}

impl NvList {
    pub fn new(nvflag: u32) -> NvList {
        NvList {
            version: NV_VERSION,
            nvflag: nvflag,
            pairs: HashMap::new(),
        }
    }
}
