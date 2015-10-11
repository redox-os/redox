const MZAP_ENT_LEN: usize = 64;
const MZAP_NAME_LEN: usize = MZAP_ENT_LEN - 8 - 4 - 2;

#[repr(u64)]
pub enum ZapObjectType {
    Micro = (1 << 63) + 3,
    Header = (1 << 63) + 1,
    Leaf = 1 << 63,
}

/// Microzap
pub struct MZapPhys {
    block_type: u64, // ZapObjectType::Micro
    salt: u64,
    normflags: u64,
    pad: [u64; 5],
    chunk: [MZapEntPhys; 1],
    // actually variable size depending on block size
}

pub struct MZapEntPhys{
    pub value: u64,
    pub cd: u32,
    pub pad: u16,
    pub name: [u8; MZAP_NAME_LEN],
}

/// Fatzap
pub struct ZapPhys {
    pub block_type: u64, // ZapObjectType::Header
    pub magic: u64,
    pub ptr_table: ZapTablePhys,
    pub free_block: u64,
    pub num_leafs: u64,
    pub num_entries: u64,
    pub salt: u64,
    pub pad: [u64; 8181],
    pub leafs: [u64; 8192],
}

pub struct ZapTablePhys {
     pub block: u64,
     pub num_blocks: u64,
     pub shift: u64,
     pub next_block: u64,
     pub block_copied: u64,
}

const ZAP_LEAF_MAGIC: u32 = 0x2AB1EAF;
const ZAP_LEAF_CHUNKSIZE: usize = 24;

// The amount of space within the chunk available for the array is:
// chunk size - space for type (1) - space for next pointer (2)
const ZAP_LEAF_ARRAY_BYTES: usize = ZAP_LEAF_CHUNKSIZE - 3;

/*pub struct ZapLeafPhys {
    pub header: ZapLeafHeader,
    hash: [u16; ZAP_LEAF_HASH_NUMENTRIES],
    union zap_leaf_chunk {
        entry,
        array,
        free,
    } chunks[ZapLeafChunk; ZAP_LEAF_NUMCHUNKS],
}*/

pub struct ZapLeafHeader {
    pub block_type: u64, // ZapObjectType::Leaf
    pub next: u64,
    pub prefix: u64,
    pub magic: u32,
    pub n_free: u16,
    pub n_entries: u16,
    pub prefix_len: u16,
    pub free_list: u16,
    pad2: [u8; 12],
}

struct ZapLeafEntry {
    leaf_type: u8,
    int_size: u8,
    next: u16,
    name_chunk: u16,
    name_length: u16,
    value_chunk: u16,
    value_length: u16,
    cd: u16,
    pad: [u8; 2],
    hash: u64,
}

struct ZapLeafArray {
    leaf_type: u8,
    array: [u8; ZAP_LEAF_ARRAY_BYTES],
    next: u16,
}

struct ZapLeafFree{
    free_type: u8,
    pad: [u8; ZAP_LEAF_ARRAY_BYTES],
    next: u16,
}
