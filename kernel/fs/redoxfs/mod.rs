use alloc::boxed::Box;

use collections::borrow::ToOwned;
use collections::string::{String, ToString};
use collections::vec::Vec;

use common::slice::GetSlice;
use arch::memory::Memory;

use core::{cmp, ptr};

use disk::Disk;

use system::error::{Error, Result, EINVAL};

pub use self::header::Header;
pub use self::node::{Node, NodeData};

pub mod header;
pub mod node;

/// A file system
pub struct FileSystem {
    pub disk: Box<Disk>,
    pub header: Header,
    pub nodes: Vec<Node>,
}

impl FileSystem {
    /// Create a file system from a disk
    pub fn from_disk(mut disk: Box<Disk>) -> Result<Self> {
        let mut header_data = try!(Memory::<u8>::new(512));
        try!(disk.read(1, header_data.as_mut_slice()));

        let header = unsafe { ptr::read(header_data.as_ptr() as *const Header) };
        if header.valid() {
            debugln!("{}: Redox Filesystem", disk.name());

            let mut nodes = Vec::new();
            for extent in &header.extents {
                if extent.block > 0 && extent.length > 0 {
                    let current_sectors = (extent.length as usize + 511) / 512;
                    let max_size = current_sectors * 512;

                    let size = cmp::min(extent.length as usize, max_size);

                    let mut buffer = try!(Memory::<u8>::new(max_size));
                    try!(disk.read(extent.block, buffer.as_mut_slice()));

                    for i in 0..size / 512 {
                        nodes.push(Node::new(extent.block + i as u64, unsafe {
                            &*(buffer.as_ptr().offset(i as isize * 512) as *const NodeData)
                        }));
                    }
                }
            }

            Ok(FileSystem {
                disk: disk,
                header: header,
                nodes: nodes,
            })
        } else {
            debugln!("{}: Unknown Filesystem", disk.name());
            Err(Error::new(EINVAL))
        }
    }

    /// Get node with a given filename
    pub fn node(&self, filename: &str) -> Option<Node> {
        for node in self.nodes.iter() {
            if node.name == filename {
                return Some(node.clone());
            }
        }

        None
    }

    /// List nodes in a given directory
    pub fn list(&self, directory_str: &str) -> Vec<String> {
        let mut ret = Vec::new();

        let directory = if directory_str.is_empty() {
            directory_str.to_owned()
        } else {
            directory_str.to_owned() + "/"
        };

        for node in self.nodes.iter() {
            if node.name.starts_with(&directory) {
                ret.push(node.name.get_slice(directory.len()..).to_string());
            }
        }

        ret
    }
}
