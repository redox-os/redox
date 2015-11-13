pub enum Data {
    File(File),
    Dir(Dir),
}

pub struct File {
    name: String,
    data: Vec<u8>,
}

impl File {
    pub fn from_bytes(b: &[u8]) -> Self {
        let name = unsafe {
            String::from_utf8_unchecked(b[0..64].to_vec())
        };
        let data = b[257..].to_vec();

        File {
            name: name,
            data: data,
        }
    }
}

pub struct Dir {
    name: String,
    data: Vec<u8>,
}

impl Dir {
    pub fn from_bytes(b: &[u8]) -> Self {
        let name = unsafe {
            String::from_utf8_unchecked(b[0..64].to_vec())
        };
        let mut n = 0;
        while let Some(35) = b.get(n + 256 - 1) {
            n += 256;
        }

        let data = b[n..].to_vec();

        Dir {
            name: name,
            data: data,
        }
    }
}
