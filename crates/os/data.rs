use redox::prelude::v1::*;
use table::NodeTable;

/// A data node (file/dir)
pub enum Data {
    /// File
    File(File),
    /// Directory
    Dir(Dir),
    /// Nothing
    Nil,
}

impl Data {
    pub fn name(&self) -> &str {
        match self {
            &Data::File(ref f) => &f.name,
            &Data::Dir(ref d) => &d.name,
            &Data::Nil => "\0",
        }
    }
}

/// A file
pub struct File {
    /// The name of the file
    name: String,
    /// The actual content of the file
    data: Vec<u8>,
}

impl File {
    /// Create a file from a slice of bytes
    pub fn from_bytes(b: &[u8]) -> Self {
        let name = unsafe {
            String::from_utf8_unchecked(b[0..64].to_vec())
        };
        let data = b[256..].to_vec();

        File {
            name: name,
            data: data,
        }
    }
}

/// A directory
pub struct Dir {
    /// The name of the directory
    name: String,
    /// The table of the directory
    nodes: Vec<DataPtr>,
}

impl Dir {
    /// Create a new directory from a slice of bytes
    pub fn from_bytes(b: &[u8]) -> Self {
        let name = unsafe {
            String::from_utf8_unchecked(b[0..64].to_vec())
        };
        let mut n = 0;
        while let Some(&35) = b.get(n + 256 - 1) {
            n += 256;
        }

        let nodes = b[n..].to_vec().iter().splitn(16).map(|x| DataPtr::from_bytes(x)).collect();

        Dir {
            name: name,
            nodes: nodes,
        }
    }

    /// Get the table represented by this directory
    pub fn get_table<'a>(&'a self) -> NodeTable<'a> {
        NodeTable::from_bytes(&self.data[..])
    }
}
