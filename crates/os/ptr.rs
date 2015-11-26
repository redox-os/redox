use archive::Archive;
use data::{Data, File, Dir};

/// An data pointer
pub enum DataPtr {
    /// A pointer pointing to an file
    File(FilePtr),
    /// A pointer pointing to an dir
    Dir(DirPtr),
}

impl DataPtr {
    /// Create an pointer from a slice of 16 bytes
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() != 16 {
            None
        } else {
            let data_type = b[0];
            let pos = b[1..9].iter().fold(0, |x, &i| x << 8 | i as u64);
            let len = b[9..16].iter().fold(0, |x, &i| x << 8 | i as u64);

            match data_type {
                102 => {
                    // File
                    Some(DataPtr::File(FilePtr {
                        pos: pos,
                        len: len,
                    }))
                }
                100 => {
                    Some(DataPtr::Dir(DirPtr {
                        pos: pos,
                        len: len,
                    }))
                }
                _ => None,
            }
        }
    }

    /// Deref this pointer to a data from the archive
    pub fn deref(&self, ar: &Archive) -> Data {
        match self {
            &DataPtr::File(ref fptr) => fptr.deref(ar),
            &DataPtr::Dir(ref dptr) => dptr.deref(ar),
        }
    }
}

/// A pointer to a file archive
pub struct FilePtr {
    /// Position
    pos: u64,
    /// Length
    len: u64,
}

impl FilePtr {
    /// Deref this pointer to a block of data from the archive
    pub fn deref(&self, ar: &Archive) -> Data {
        Data::File(File::from_bytes(&ar.files[self.pos as usize..(self.pos + self.len) as usize]))
    }
}

/// A pointer to a directory
pub struct DirPtr {
    /// Position
    pos: u64,
    /// Length (per 16 bytes)
    len: u64,
}

impl DirPtr {
    /// Deref this pointer to a block of data from the archive
    pub fn deref(&self, ar: &Archive) -> Data {
        Data::Dir(Dir::from_bytes(&ar.files[self.pos as usize..(self.pos + self.len) as usize]))
    }
}
