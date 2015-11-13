use data::{Data, DataType};
use file::File;
use dir::Dir;
use archive::Archive;

pub struct DataPtr {
    pub data_type: DataType,
    pos: [u8; 5],
    len: u16,
    offset: usize,
}

impl DataPtr {
    pub fn from_bytes(offset: usize, offset: usize, b: &[u8]) -> Self {
        DataPtr {
            data_type: DataType::from_byte(b[0]),
            pos: [b[1], b[2], b[3], b[4], b[5]],
            len: b[6] << 8 | b[7],
            offset: offset,
        }
    }

    pub fn pos(&self) -> usize {
        (((((((((pos[0] << 8) | pos[1]) << 8) | pos[2]) << 8) | pos[3]) << 8) | pos[4]) as usize) * 64 + offset
    }

    pub fn len(&self) -> usize {
        self.len as usize * 32
    }

    pub fn deref(&'a self, ar: &'a Archive) -> Option<Data<'a>> {
        match self.data_type {
            DataType::File => {
                Some(
                    Data::Dir(File::from_bytes(
                        ar.data[ar.offset + self.pos()..ar.offset + self.pos() + self.len()]
                    ))
                )
            },
            DataType::Dir => {
                Some(
                    Data::Dir(Dir::from_bytes(
                        ar.data[ar.offset + self.pos()..ar.offset + self.pos() + self.len()]
                    ))
                )
            },
            _ => None,
        }
    }
}
