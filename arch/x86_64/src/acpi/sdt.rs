use core::mem;

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct SDT {
  pub signature: [u8; 4],
  pub length: u32,
  pub revision: u8,
  pub checksum: u8,
  pub oem_id: [u8; 6],
  pub oem_table_id: [u8; 8],
  pub oem_revision: u32,
  pub creator_id: u32,
  pub creator_revision: u32
}

impl SDT {
    /// Get the address of this tables data
    pub fn data_address(&'static self) -> usize {
        self as *const _ as usize + mem::size_of::<SDT>()
    }

    /// Get the length of this tables data
    pub fn data_len(&'static self) -> usize {
        let total_size = self.length as usize;
        let header_size = mem::size_of::<SDT>();
        if total_size >= header_size {
            total_size - header_size
        } else {
            0
        }
    }
}
