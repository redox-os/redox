use core::clone::Clone;
use core::mem::size_of;
use core::option::Option;

use common::memory::*;
use common::safeptr::*;
use common::string::*;
use common::vector::*;

use drivers::disk::*;

#[derive(Copy, Clone)]
pub struct Block {
    address: u64
}

#[derive(Copy, Clone)]
struct Extent{
    block: Block,
    length: u64
}

pub struct Header {
    pub signature: [u8; 4],
    pub version: u32,
    pub root_sector_list: Block,
    pub free_sector_list: Block,
    pub name: [u8; 256],
    reserved: [u8; 232]
}

struct Node {
    parent_collection: Block,
    data_sector_list: Block,
    data_size: u64,
    user_id: u64,
    group_id: u64,
    mode: u64,
    create_time: u64,
    modify_time: u64,
    access_time: u64,
    name: [u8; 256],
    reserved: [u8; 184]
}

struct SectorList {
    parent_node: Block,
    fragment_number: u64,
    last_fragment: Block,
    next_fragment: Block,
    extents: [Extent; 30]
}

pub struct UnFS {
    disk: Disk,
    pub header: &'static Header
}

impl UnFS {
    pub fn new() -> UnFS{
        unsafe{
            return UnFS::from_disk(Disk::new());
        }
    }

    pub unsafe fn from_disk(disk: Disk) -> UnFS{
        // TODO: Do not use header loaded in memory
        UnFS { disk:disk, header: &*(0x7E00 as *const Header) }
    }

    pub unsafe fn node(&self, filename: String) -> *const Node{
        let mut ret: *const Node = 0 as *const Node;
        let mut node_matches = false;

        let root_sector_list = alloc(size_of::<SectorList>()) as *const SectorList;
        let mut root_sector_list_address = self.header.root_sector_list.address;
        while root_sector_list_address > 0 {
            self.disk.read(root_sector_list_address, 1, root_sector_list as usize);

            for extent_i in 0..30 {
                let extent = (*root_sector_list).extents[extent_i];
                if extent.block.address > 0 {
                    for node_address in extent.block.address..extent.block.address + extent.length {
                        let node = alloc(size_of::<Node>()) as *const Node;
                        self.disk.read(node_address, 1, node as usize);

                        node_matches = true;
                        let mut i = 0;
                        for c in filename.chars()  {
                            if !(i < 256 && (*node).name[i] == c as u8) {
                                node_matches = false;
                                break;
                            }
                            i += 1;
                        }
                        if !(i < 256 && (*node).name[i] == 0) {
                            node_matches = false;
                        }

                        unalloc(node as usize);

                        if node_matches {
                            ret = node;
                            break;
                        }
                    }
                }

                if node_matches {
                    break;
                }
            }

            root_sector_list_address = (*root_sector_list).next_fragment.address;

            if node_matches{
                break;
            }
        }

        unalloc(root_sector_list as usize);

        ret
    }

    pub fn list(&self, directory: String) -> Vector<String> {
        let mut ret = Vector::<String>::new();

        unsafe{
            let root_sector_list = alloc(size_of::<SectorList>()) as *const SectorList;
            let mut root_sector_list_address = self.header.root_sector_list.address;
            while root_sector_list_address > 0 {
                self.disk.read(root_sector_list_address, 1, root_sector_list as usize);

                for extent_i in 0..30 {
                    let extent = (*root_sector_list).extents[extent_i];
                    if extent.block.address > 0 {
                        for node_address in extent.block.address..extent.block.address + extent.length {
                            let node = alloc(size_of::<Node>()) as *const Node;
                            self.disk.read(node_address, 1, node as usize);

                            let node_name = String::from_c_slice(&(*node).name);
                            if node_name.starts_with(directory.clone()) {
                                ret = ret + node_name;
                            }

                            unalloc(node as usize);
                        }
                    }
                }

                root_sector_list_address = (*root_sector_list).next_fragment.address;
            }

            unalloc(root_sector_list as usize);
        }

        ret
    }

    pub unsafe fn load(&self, filename: String) -> usize{
        let mut destination = 0;

        let node = self.node(filename);

        if node as usize > 0{
            if (*node).data_sector_list.address > 0 {
                let sector_list_ptr: SafePtr<SectorList> = SafePtr::new();
                match sector_list_ptr.get() {
                    Option::Some(sector_list) => {
                        self.disk.read((*node).data_sector_list.address, 1, sector_list_ptr.unsafe_ptr() as usize);

                        //TODO: More than one extent, extent sector count > 64K
                        let mut size = 0;
                        for i in 0..1 {
                            if sector_list.extents[i].block.address > 0 && sector_list.extents[i].length > 0{
                                size += (*sector_list).extents[i].length * 512;
                            }
                        }

                        destination = alloc(size as usize);
                        if destination > 0 {
                            for i in 0..1 {
                                if sector_list.extents[i].block.address > 0 && sector_list.extents[i].length > 0{
                                    self.disk.read(sector_list.extents[i].block.address, sector_list.extents[i].length as u16, destination);
                                }
                            }
                        }
                    },
                    Option::None => ()
                }
            }

            unalloc(node as usize);
        }

        return destination;
    }

    // TODO: Support realloc of LBAs
    pub unsafe fn save(&self, filename: String, source: usize){
        let node = self.node(filename);

        if node as usize > 0{
            if (*node).data_sector_list.address > 0 {
                let sector_list_ptr: SafePtr<SectorList> = SafePtr::new();
                match sector_list_ptr.get() {
                    Option::Some(sector_list) => {
                        self.disk.read((*node).data_sector_list.address, 1, sector_list_ptr.unsafe_ptr() as usize);

                        if source > 0 {
                            for i in 0..1 {
                                if sector_list.extents[i].block.address > 0 && sector_list.extents[i].length > 0{
                                    self.disk.write(sector_list.extents[i].block.address, sector_list.extents[i].length as u16, source);
                                }
                            }
                        }
                    },
                    Option::None => ()
                }
            }

            unalloc(node as usize);
        }
    }
}
