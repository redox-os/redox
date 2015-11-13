use archive::Archive;

pub enum DataPtr {
    File(FilePtr),
    Dir(DirPtr),
}

impl DataPtr {
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() != 80 {
            None
        } else {
            let data_type = b[0];
            let pos = b[1..9].iter().fold(0, |x, &i| x << 8 | i as u64);
            let len = b[9..17].iter().fold(0, |x, &i| x << 8 | i as u64);

            match data_type {
                102 => { // File
                    Some(DataPtr::File(
                        FilePtr {
                            pos: pos,
                            len: len,
                        }
                    ))
                },
                100 => {
                    Some(DataPtr::Dir(
                        DirPtr {
                            pos: pos,
                            len: len,
                        }
                    ))
                },
                _ => None,
            }
        }
    }

    pub fn deref(&self, ar: &Archive) -> Data {
        match self {
            File(fptr) => fptr.deref(ar),
            Dir(dptr) => dptr.deref(ar),
        }
    }
}

pub struct FilePtr {
    pos: u64,
    len: u64,
}

impl FilePtr {
    pub fn deref(&self, ar: &Archive) -> Data {
        Data::File(File::from_bytes(&ar.files[self.pos..self.pos + self.len]))
    }
}

pub struct DirPtr {
    pos: u64,
    len: u64,
}

impl DirPtr {
    pub fn deref(&self, ar: &Archive) -> Data {
        Data::Dir(Dir::from_bytes(&ar.files[self.pos..self.pos + self.len]))
    }
}
