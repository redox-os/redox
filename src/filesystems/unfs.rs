use core::clone::Clone;
use core::mem::size_of;
use core::ptr;

use common::memory::*;
use common::string::*;
use common::vec::*;

use drivers::disk::*;

#[derive(Copy, Clone)]
#[repr(packed)]
pub struct Extent{
    pub block: u64,
    pub length: u64
}

#[repr(packed)]
pub struct Header {
    pub signature: [u8; 4],
    pub version: u32,
    pub name: [u8; 248],
    pub extents: [Extent; 16]
}

#[repr(packed)]
pub struct Node {
    pub name: [u8; 256],
    pub extents: [Extent; 16]
}

impl Clone for Node {
    fn clone(&self) -> Node {
        return Node {
            name: self.name,
            extents: self.extents
        };
    }
}

pub struct UnFS {
    pub disk: Disk,
    pub header: Header,
    pub nodes: Vec<Node>
}

impl UnFS {
    pub fn from_disk(disk: Disk) -> UnFS{
        unsafe{
            let header_ptr: *const Header = alloc_type();
            disk.read(1, 1, header_ptr as usize);
            let header = ptr::read(header_ptr);
            unalloc(header_ptr as usize);

            let mut nodes = Vec::new();
            let node_ptr: *const Node = alloc_type();
            for extent in &header.extents {
                if extent.block > 0 {
                    for node_address in extent.block..extent.block + (extent.length + 511)/512 {
                        disk.read(node_address, 1, node_ptr as usize);
                        nodes.push(ptr::read(node_ptr));
                    }
                }
            }
            unalloc(node_ptr as usize);

            return UnFS {
                disk:disk,
                header: header,
                nodes: nodes
            };
        }
    }

    pub fn valid(&self) -> bool {
        return self.header.signature[0] == 'U' as u8
            && self.header.signature[1] == 'n' as u8
            && self.header.signature[2] == 'F' as u8
            && self.header.signature[3] == 'S' as u8
            && self.header.version == 0xFFFFFFFF;
    }

    pub fn node(&self, filename: &String) -> Option<Node>{
        for node in self.nodes.iter() {
            if String::from_c_slice(&node.name) == *filename {
                return Option::Some(node.clone());
            }
        }

        return Option::None;
    }

    pub fn list(&self, directory: &String) -> Vec<String> {
        let mut ret = Vec::<String>::new();

        for node in self.nodes.iter() {
            let node_name = String::from_c_slice(&node.name);
            if directory.len() > 0 {
                if node_name.starts_with(directory.clone() + "/") {
                    ret.push(node_name.substr(directory.len() + 1, node_name.len() - directory.len() - 1));
                }
            }else{
                ret.push(node_name);
            }
        }

        return ret;
    }

    // TODO: Support realloc of LBAs and save function
}
