use core::clone::Clone;
use core::mem::size_of;
use core::ptr;

use common::memory::*;
use common::string::*;
use common::vec::*;

use drivers::disk::*;

#[derive(Copy, Clone)]
pub struct Extent{
    pub block: u64,
    pub length: u64
}

pub struct Header {
    pub signature: [u8; 4],
    pub version: u32,
    pub name: [u8; 248],
    pub extents: [Extent; 16]
}

pub struct Node {
    pub name: [u8; 256],
    pub extents: [Extent; 16]
}

pub struct UnFS {
    pub disk: Disk,
    pub header: Header
}

impl UnFS {
    pub fn from_disk(disk: Disk) -> UnFS{
        unsafe{
            let header_ptr: *const Header = alloc_type();
            disk.read(1, 1, header_ptr as usize);
            let ret = UnFS { disk:disk, header: ptr::read(header_ptr) };
            unalloc(header_ptr as usize);
            return ret;
        }
    }

    pub fn valid(&self) -> bool {
        return self.header.signature[0] == 'U' as u8
            && self.header.signature[1] == 'n' as u8
            && self.header.signature[2] == 'F' as u8
            && self.header.signature[3] == 'S' as u8
            && self.header.version == 0xFFFFFFFF;
    }

    pub fn node(&self, filename: String) -> Option<Node>{
        let mut ret: Option<Node> = Option::None;

        unsafe{
            let node_ptr: *const Node = alloc_type();
            if node_ptr as usize > 0 {
                for extent in &self.header.extents {
                    if extent.block > 0 {
                        for node_address in extent.block..extent.block + extent.length {
                            self.disk.read(node_address, 1, node_ptr as usize);

                            if String::from_c_slice(&(*node_ptr).name) == filename {
                                ret = Option::Some(ptr::read(node_ptr));
                                break;
                            }
                        }
                    }

                    if ret.is_some() {
                        break;
                    }
                }
                unalloc(node_ptr as usize);
            }
        }

        return ret;
    }

    pub fn list(&self, directory: String) -> Vec<String> {
        let mut ret = Vec::<String>::new();

        unsafe{
            let node_ptr: *const Node = alloc_type();
            if node_ptr as usize > 0 {
                for extent in &self.header.extents {
                    if extent.block > 0 {
                        for node_address in extent.block..extent.block + extent.length {
                            self.disk.read(node_address, 1, node_ptr as usize);

                            let node_name = String::from_c_slice(&(*node_ptr).name);
                            if node_name.starts_with(directory.clone()) {
                                ret.push(node_name);
                            }
                        }
                    }
                }
                unalloc(node_ptr as usize);
            }
        }

        ret
    }

    // TODO: Support realloc of LBAs and save function
}
