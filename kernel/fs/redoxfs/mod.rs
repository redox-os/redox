use collections::string::{String, ToString};
use collections::vec::Vec;

use common::get_slice::GetSlice;
use common::memory::Memory;

use core::mem;

use disk::ide::Disk;

pub use self::header::Header;
pub use self::node::{Node, NodeData};

pub mod header;
pub mod node;

/// A file system
pub struct FileSystem {
    pub disk: Disk,
    pub header: Header,
    pub nodes: Vec<Node>,
}

impl FileSystem {
    /// Create a file system from a disk
    pub fn from_disk(disk: Disk) -> Option<Self> {
        unsafe {
            if disk.identify() {
                debug!(" Disk Found");

                let header_ptr = Memory::<Header>::new(1).unwrap();
                disk.read(1, 1, header_ptr.address());
                let header = header_ptr.read(0);
                drop(header_ptr);

                if header.valid() {
                    debugln!(" Redox Filesystem");

                    let mut nodes = Vec::new();
                    for extent in &header.extents {
                        if extent.block > 0 && extent.length > 0 {
                            if let Some(data) =
                                   Memory::<NodeData>::new(extent.length as usize /
                                                           mem::size_of::<NodeData>()) {
                                let sectors = (extent.length as usize + 511) / 512;
                                let mut sector: usize = 0;
                                while sectors - sector >= 65536 {
                                    disk.read(extent.block + sector as u64,
                                              0,
                                              data.address() + sector * 512);

                                    //
                                    // let request = Request {
                                    // extent: Extent {
                                    // block: extent.block + sector as u64,
                                    // length: 65536 * 512,
                                    // },
                                    // mem: data.address() + sector * 512,
                                    // read: true,
                                    // complete: Arc::new(AtomicBool::new(false)),
                                    // };
                                    //
                                    // disk.request(request.clone());
                                    //
                                    // while request.complete.load(Ordering::SeqCst) == false {
                                    // disk.on_poll();
                                    // }
                                    //

                                    sector += 65535;
                                }
                                if sector < sectors {
                                    disk.read(extent.block + sector as u64,
                                              (sectors - sector) as u16,
                                              data.address() + sector * 512);
                                    //
                                    // let request = Request {
                                    // extent: Extent {
                                    // block: extent.block + sector as u64,
                                    // length: (sectors - sector) as u64 * 512,
                                    // },
                                    // mem: data.address() + sector * 512,
                                    // read: true,
                                    // complete: Arc::new(AtomicBool::new(false)),
                                    // };
                                    //
                                    // disk.request(request.clone());
                                    //
                                    // while request.complete.load(Ordering::SeqCst) == false {
                                    // disk.on_poll();
                                    // }
                                    //
                                }

                                for i in 0..extent.length as usize / mem::size_of::<NodeData>() {
                                    nodes.push(Node::new(extent.block + i as u64, &data[i]));
                                }
                            }
                        }
                    }

                    return Some(FileSystem {
                        disk: disk,
                        header: header,
                        nodes: nodes,
                    });
                } else {
                    debugln!(" Unknown Filesystem");
                }
            } else {
                debugln!(" Disk Not Found");
            }
        }

        None
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
    pub fn list(&self, directory: &str) -> Vec<String> {
        let mut ret = Vec::new();

        for node in self.nodes.iter() {
            if node.name.starts_with(directory) {
                ret.push(node.name.get_slice(Some(directory.len()), None).to_string());
            }
        }

        ret
    }
}
