use collections::string::String;
use collections::vec::Vec;

use disk::ide::Extent;

/// Data for a node
#[repr(packed)]
pub struct NodeData {
    pub name: [u8; 256],
    pub extents: [Extent; 16],
}

/// A file node
pub struct Node {
    pub block: u64,
    pub name: String,
    pub extents: [Extent; 16],
}

impl Node {
    /// Create a new file node from an address and some data
    pub fn new(block: u64, data: &NodeData) -> Self {
        let mut bytes = Vec::new();
        for b in data.name.iter() {
            if *b > 0 {
                bytes.push(*b);
            } else {
                break;
            }
        }

        Node {
            block: block,
            name: unsafe { String::from_utf8_unchecked(bytes) },
            extents: data.extents,
        }
    }

    pub fn data(&self) -> NodeData {
        let mut name: [u8; 256] = [0; 256];
        let mut i = 0;
        for b in self.name.as_bytes().iter() {
            if i < name.len() {
                name[i] = *b;
            } else {
                break;
            }
            i += 1;
        }
        NodeData {
            name: name,
            extents: self.extents,
        }
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Node {
            block: self.block,
            name: self.name.clone(),
            extents: self.extents,
        }
    }
}
