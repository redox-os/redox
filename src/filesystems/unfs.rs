use core::ptr;
use core::str::StrExt;

use common::memory::*;

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

impl Node {
    fn empty() -> Node{
        Node{
            parent_collection: Block{ address:0 },
            data_sector_list: Block{ address:0 },
            data_size: 0,
            user_id: 0,
            group_id: 0,
            mode: 0,
            create_time: 0,
            modify_time: 0,
            access_time: 0,
            name: [0; 256],
            reserved: [0; 184]
        }
    }
}

struct SectorList {
    parent_node: Block,
    fragment_number: u64,
    last_fragment: Block,
    next_fragment: Block,
    extents: [Extent; 30]
}

impl SectorList {
    fn empty() -> SectorList{
        SectorList{
            parent_node: Block{ address:0 },
            fragment_number: 0,
            last_fragment: Block{ address:0 },
            next_fragment: Block{ address:0 },
            extents: [Extent{ block:Block{ address:0 }, length:0 }; 30]
        }
    }
}

pub struct UnFS {
    disk: Disk,
    pub header: &'static Header
}

impl UnFS {
    pub unsafe fn new(disk: Disk) -> UnFS{
        UnFS { disk:disk, header: &*(0x7E00 as *const Header) }
    }

    pub unsafe fn node(&self, filename: &str) -> *const Node{
        let mut ret: *const Node = ptr::null();
        let mut node_matches = false;

        let mut root_sector_list_address = self.header.root_sector_list.address;
        while root_sector_list_address > 0 {
            self.disk.read(root_sector_list_address, 1, 0x200400);
            let root_sector_list = &*(0x200400 as *const SectorList);

            for extent_i in 0..30 {
                let extent = root_sector_list.extents[extent_i];
                if extent.block.address > 0 {
                    for node_address in extent.block.address..extent.block.address + extent.length {
                        self.disk.read(node_address, 1, 0x200800);
                        let node = 0x200800 as *const Node;

                        node_matches = true;
                        let mut i = 0;
                        for character in filename.chars()  {
                            if !(i < 256 && (*node).name[i] == character as u8) {
                                node_matches = false;
                                break;
                            }
                            i += 1;
                        }
                        if !(i < 256 && (*node).name[i] == 0) {
                            node_matches = false;
                        }
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

            root_sector_list_address = root_sector_list.next_fragment.address;

            if node_matches{
                break;
            }
        }

        ret
    }

    pub unsafe fn load(&self, filename: &str) -> usize{
        let node = self.node(filename);
        
        if node != ptr::null() && (*node).data_sector_list.address > 0 {
            self.disk.read((*node).data_sector_list.address, 1, 0x200B00);
            let sector_list = &*(0x200B00 as *const SectorList);

            //TODO: More than one extent, extent sector count > 64K
            let mut size = 0;
            for i in 0..1 {
                if sector_list.extents[i].block.address > 0 && sector_list.extents[i].length > 0{
                    size += sector_list.extents[i].length * 512;
                }
            }
            
            let destination = alloc(size as usize);
            if destination > 0 {
                for i in 0..1 {
                    if sector_list.extents[i].block.address > 0 && sector_list.extents[i].length > 0{
                        self.disk.read(sector_list.extents[i].block.address, sector_list.extents[i].length as u16, destination);
                    }
                }
            }
            return destination;
        }
        return 0;
    }
}