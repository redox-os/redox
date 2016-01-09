use std::fmt;

// nvp implementation version
pub const NV_VERSION: i32 = 0;

// nvlist header
// #[derive(Debug)]
pub struct NvList {
    pub version: i32,
    pub nvflag: u32, // persistent flags
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

    pub fn add(&mut self, name: String, value: NvValue) {
        self.pairs.push((name, value));
    }

    pub fn find(&self, name: &str) -> Option<&NvValue> {
        for pair in &self.pairs {
            if pair.0 == name {
                return Some(&pair.1);
            }
        }
        None
    }

    pub fn find_mut(&mut self, name: &str) -> Option<&mut NvValue> {
        for pair in &mut self.pairs {
            if pair.0 == name {
                return Some(&mut pair.1);
            }
        }
        None
    }

    pub fn get<'a, T: GetNvValue<'a>>(&'a self, name: &str) -> Option<T> {
        self.find(name).and_then(|x| GetNvValue::get(x))
    }
}

impl fmt::Debug for NvList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f,
                    "NvList {{ version: {:X}, nvflag: {:X}, pairs: [\n",
                    self.version,
                    self.nvflag));
        for &(ref name, ref value) in &self.pairs {
            if name.is_empty() {
                break;
            }
            try!(write!(f, "{} : {:?}\n", name, value));
        }
        try!(write!(f, "] }}\n"));
        Ok(())
    }
}

// TODO Auto implement Debug. format! currently crashes with big u32 values
// #[derive(Debug)]
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
    HrTime(i64),
    NvList(NvList),
    NvListArray(Vec<NvList>),
    BooleanValue(bool),
    Int8(i8),
    Uint8(u8),
    BooleanArray(Vec<bool>),
    Int8Array(Vec<i8>),
    Uint8Array(Vec<u8>),
}

impl NvValue {
    pub fn data_type(&self) -> DataType {
        match *self {
            NvValue::Unknown => DataType::Unknown,
            NvValue::Boolean => DataType::Boolean,
            NvValue::Byte(_) => DataType::Byte,
            NvValue::Int16(_) => DataType::Int16,
            NvValue::Uint16(_) => DataType::Uint16,
            NvValue::Int32(_) => DataType::Int32,
            NvValue::Uint32(_) => DataType::Uint32,
            NvValue::Int64(_) => DataType::Int64,
            NvValue::Uint64(_) => DataType::Uint64,
            NvValue::String(_) => DataType::String,
            NvValue::ByteArray(_) => DataType::ByteArray,
            NvValue::Int16Array(_) => DataType::Int16Array,
            NvValue::Uint16Array(_) => DataType::Uint16Array,
            NvValue::Int32Array(_) => DataType::Int32Array,
            NvValue::Uint32Array(_) => DataType::Uint32Array,
            NvValue::Int64Array(_) => DataType::Int64Array,
            NvValue::Uint64Array(_) => DataType::Uint64Array,
            NvValue::StringArray(_) => DataType::StringArray,
            NvValue::HrTime(_) => DataType::HrTime,
            NvValue::NvList(_) => DataType::NvList,
            NvValue::NvListArray(_) => DataType::NvListArray,
            NvValue::BooleanValue(_) => DataType::BooleanValue,
            NvValue::Int8(_) => DataType::Int8,
            NvValue::Uint8(_) => DataType::Uint8,
            NvValue::BooleanArray(_) => DataType::BooleanArray,
            NvValue::Int8Array(_) => DataType::Int8Array,
            NvValue::Uint8Array(_) => DataType::Uint8Array,
        }
    }

    pub fn num_elements(&self) -> usize {
        match *self {
            NvValue::Unknown => 1,
            NvValue::Boolean => 1,
            NvValue::Byte(_) => 1,
            NvValue::Int16(_) => 1,
            NvValue::Uint16(_) => 1,
            NvValue::Int32(_) => 1,
            NvValue::Uint32(_) => 1,
            NvValue::Int64(_) => 1,
            NvValue::Uint64(_) => 1,
            NvValue::String(_) => 1,
            NvValue::ByteArray(ref a) => a.len(),
            NvValue::Int16Array(ref a) => a.len(),
            NvValue::Uint16Array(ref a) => a.len(),
            NvValue::Int32Array(ref a) => a.len(),
            NvValue::Uint32Array(ref a) => a.len(),
            NvValue::Int64Array(ref a) => a.len(),
            NvValue::Uint64Array(ref a) => a.len(),
            NvValue::StringArray(ref a) => a.len(),
            NvValue::HrTime(_) => 1,
            NvValue::NvList(_) => 1,
            NvValue::NvListArray(ref a) => a.len(),
            NvValue::BooleanValue(_) => 1,
            NvValue::Int8(_) => 1,
            NvValue::Uint8(_) => 1,
            NvValue::BooleanArray(ref a) => a.len(),
            NvValue::Int8Array(ref a) => a.len(),
            NvValue::Uint8Array(ref a) => a.len(),
        }
    }
}

impl fmt::Debug for NvValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NvValue::Int64(v) => write!(f, "Int64(0x{:X})", v),
            NvValue::Uint64(v) => write!(f, "Uint64(0x{:X})", v),
            NvValue::NvList(ref v) => write!(f, "NvList({:?})", v),
            NvValue::NvListArray(ref v) => {
                try!(write!(f, "NvListArray(["));
                for nv_list in v {
                    try!(write!(f, "NvList({:?})", nv_list));
                }
                write!(f, "])")
            }
            NvValue::String(ref v) => write!(f, "String({})", v),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum DataType {
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

impl DataType {
    pub fn from_u8(u: u8) -> Option<DataType> {
        match u {
            0 => Some(DataType::Unknown),
            1 => Some(DataType::Boolean),
            2 => Some(DataType::Byte),
            3 => Some(DataType::Int16),
            4 => Some(DataType::Uint16),
            5 => Some(DataType::Int32),
            6 => Some(DataType::Uint32),
            7 => Some(DataType::Int64),
            8 => Some(DataType::Uint64),
            9 => Some(DataType::String),
            10 => Some(DataType::ByteArray),
            11 => Some(DataType::Int16Array),
            12 => Some(DataType::Uint16Array),
            13 => Some(DataType::Int32Array),
            14 => Some(DataType::Uint32Array),
            15 => Some(DataType::Int64Array),
            16 => Some(DataType::Uint64Array),
            17 => Some(DataType::StringArray),
            18 => Some(DataType::HrTime),
            19 => Some(DataType::NvList),
            20 => Some(DataType::NvListArray),
            21 => Some(DataType::BooleanValue),
            22 => Some(DataType::Int8),
            23 => Some(DataType::Uint8),
            24 => Some(DataType::BooleanArray),
            25 => Some(DataType::Int8Array),
            26 => Some(DataType::Uint8Array),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            DataType::Unknown => 0,
            DataType::Boolean => 1,
            DataType::Byte => 2,
            DataType::Int16 => 3,
            DataType::Uint16 => 4,
            DataType::Int32 => 5,
            DataType::Uint32 => 6,
            DataType::Int64 => 7,
            DataType::Uint64 => 8,
            DataType::String => 9,
            DataType::ByteArray => 10,
            DataType::Int16Array => 11,
            DataType::Uint16Array => 12,
            DataType::Int32Array => 13,
            DataType::Uint32Array => 14,
            DataType::Int64Array => 15,
            DataType::Uint64Array => 16,
            DataType::StringArray => 17,
            DataType::HrTime => 18,
            DataType::NvList => 19,
            DataType::NvListArray => 20,
            DataType::BooleanValue => 21,
            DataType::Int8 => 22,
            DataType::Uint8 => 23,
            DataType::BooleanArray => 24,
            DataType::Int8Array => 25,
            DataType::Uint8Array => 26,
        }
    }
}

/// /////////////////////////////////////////////////////////////////////////////////////////////////

pub trait GetNvValue<'a>: Sized {
    fn get(value: &'a NvValue) -> Option<Self>;
}

impl<'a> GetNvValue<'a> for bool {
    fn get(value: &'a NvValue) -> Option<Self> {
        match *value {
            NvValue::BooleanValue(v) => Some(v),
            _ => None,
        }
    }
}

impl<'a> GetNvValue<'a> for u8 {
    fn get(value: &'a NvValue) -> Option<Self> {
        match *value {
            NvValue::Byte(v) => Some(v),
            _ => None,
        }
    }
}

impl<'a> GetNvValue<'a> for u16 {
    fn get(value: &'a NvValue) -> Option<Self> {
        match *value {
            NvValue::Uint16(v) => Some(v),
            _ => None,
        }
    }
}

impl<'a> GetNvValue<'a> for u32 {
    fn get(value: &'a NvValue) -> Option<Self> {
        match *value {
            NvValue::Uint32(v) => Some(v),
            _ => None,
        }
    }
}

impl<'a> GetNvValue<'a> for u64 {
    fn get(value: &'a NvValue) -> Option<Self> {
        match *value {
            NvValue::Uint64(v) => Some(v),
            _ => None,
        }
    }
}

impl<'a> GetNvValue<'a> for i16 {
    fn get(value: &'a NvValue) -> Option<Self> {
        match *value {
            NvValue::Int16(v) => Some(v),
            _ => None,
        }
    }
}

impl<'a> GetNvValue<'a> for i32 {
    fn get(value: &'a NvValue) -> Option<Self> {
        match *value {
            NvValue::Int32(v) => Some(v),
            _ => None,
        }
    }
}

impl<'a> GetNvValue<'a> for i64 {
    fn get(value: &'a NvValue) -> Option<Self> {
        match *value {
            NvValue::Int64(v) => Some(v),
            _ => None,
        }
    }
}

impl<'a> GetNvValue<'a> for &'a String {
    fn get(value: &'a NvValue) -> Option<Self> {
        match *value {
            NvValue::String(ref v) => Some(v),
            _ => None,
        }
    }
}

impl<'a> GetNvValue<'a> for &'a NvList {
    fn get(value: &'a NvValue) -> Option<Self> {
        match *value {
            NvValue::NvList(ref v) => Some(v),
            _ => None,
        }
    }
}

impl<'a> GetNvValue<'a> for &'a Vec<NvList> {
    fn get(value: &'a NvValue) -> Option<Self> {
        match *value {
            NvValue::NvListArray(ref v) => Some(v),
            _ => None,
        }
    }
}
