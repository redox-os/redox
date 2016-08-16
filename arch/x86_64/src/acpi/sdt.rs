#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct SDTHeader {
  pub signature: [u8; 4],
  pub length: u32,
  pub revision: u8,
  pub checksum: u8,
  pub oem_id: [u8; 6],
  pub oem_table_id: [u8; 8],
  pub oem_revision: u32,
  pub creator_id: u32,
  pub creator_revision: u32,
}
