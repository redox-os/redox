use super::block_ptr::BlockPtr;

#[repr(packed)]
pub struct ZilHeader {
    claim_txg: u64,
    replay_seq: u64,
    log: BlockPtr,
}
