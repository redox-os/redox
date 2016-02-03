use std::fmt;
use std::mem;

use super::block_ptr::BlockPtr;
use super::from_bytes::FromBytes;
use super::zil_header::ZilHeader;

#[repr(u8)]
#[derive(Debug, Eq, PartialEq)]
pub enum ObjectType {
    None,
    ObjectDirectory,
    ObjectArray,
    PackedNvList,
    NvListSize,
    BlockPtrList,
    BlockPtrListHdr,
    SpaceMapHeader,
    SpaceMap,
    IntentLog,
    DNode,
    ObjSet,
    DataSet,
    DataSetChildMap,
    ObjSetSnapMap,
    DslProps,
    DslObjSet,
    ZNode,
    Acl,
    PlainFileContents,
    DirectoryContents,
    MasterNode,
    DeleteQueue,
    ZVol,
    ZVolProp,
}

#[repr(packed)]
pub struct DNodePhys {
    pub object_type: ObjectType,
    pub indblkshift: u8, // ln2(indirect block size)
    pub nlevels: u8, // 1=blkptr->data blocks
    pub nblkptr: u8, // length of blkptr
    pub bonus_type: u8, // type of data in bonus buffer
    pub checksum: u8, // ZIO_CHECKSUM type
    pub compress: u8, // ZIO_COMPRESS type
    pub flags: u8, // DNODE_FLAG_*
    pub data_blk_sz_sec: u16, // data block size in 512b sectors
    pub bonus_len: u16, // length of bonus
    pub pad2: [u8; 4],

    // accounting is protected by dirty_mtx
    pub maxblkid: u64, // largest allocated block ID
    pub used: u64, // bytes (or sectors) of disk space

    pub pad3: [u64; 4],

    blkptr_bonus: [u8; 448],
}

impl DNodePhys {
    pub fn get_blockptr<'a>(&self, i: usize) -> &'a BlockPtr {
        unsafe { mem::transmute(&self.blkptr_bonus[i * 128]) }
    }

    pub fn get_bonus(&self) -> &[u8] {
        &self.blkptr_bonus[(self.nblkptr as usize) * 128..]
    }
}

impl FromBytes for DNodePhys {}

impl fmt::Debug for DNodePhys {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f,
                    "DNodePhys {{ object_type: {:?}, nlevels: {:X}, nblkptr: {:X}, bonus_type: \
                     {:X}, bonus_len: {:X}}}\n",
                    self.object_type,
                    self.nlevels,
                    self.nblkptr,
                    self.bonus_type,
                    self.bonus_len));
        Ok(())
    }
}
