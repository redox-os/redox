use redox::*;

use super::block_ptr::BlockPtr;
use super::dvaddr::DVAddr;
use super::from_bytes::FromBytes;
use super::lzjb;
use super::uberblock::Uberblock;

pub struct Reader {
    pub disk: File,
}

impl Reader {
    // TODO: Error handling
    pub fn read(&mut self, start: usize, length: usize) -> Vec<u8> {
        let mut ret: Vec<u8> = vec![0; length*512];

        self.disk.seek(SeekFrom::Start(start * 512));
        self.disk.read(&mut ret);

        return ret;
    }

    pub fn write(&mut self, block: usize, data: &[u8; 512]) {
        self.disk.seek(SeekFrom::Start(block * 512));
        self.disk.write(data);
    }

    pub fn read_dva(&mut self, dva: &DVAddr) -> Vec<u8> {
        self.read(dva.sector() as usize, dva.asize() as usize)
    }

    pub fn read_block(&mut self, block_ptr: &BlockPtr) -> Result<Vec<u8>, String> {
        let data = self.read_dva(&block_ptr.dvas[0]);
        match block_ptr.compression() {
            2 => {
                // compression off
                Ok(data)
            }
            1 | 3 => {
                // lzjb compression
                let mut decompressed = vec![0; (block_ptr.lsize()*512) as usize];
                lzjb::decompress(&data, &mut decompressed);
                Ok(decompressed)
            }
            _ => Err("Error: not enough bytes".to_string()),
        }
    }

    pub fn read_type<T: FromBytes>(&mut self, block_ptr: &BlockPtr) -> Result<T, String> {
        let data = self.read_block(block_ptr);
        data.and_then(|data| T::from_bytes(&data[..]))
    }

    pub fn read_type_array<T: FromBytes>(&mut self,
                                         block_ptr: &BlockPtr,
                                         offset: usize)
                                         -> Result<T, String> {
        let data = self.read_block(block_ptr);
        data.and_then(|data| T::from_bytes(&data[offset * mem::size_of::<T>()..]))
    }

    pub fn uber(&mut self) -> Result<Uberblock, String> {
        let mut newest_uberblock: Option<Uberblock> = None;
        for i in 0..128 {
            if let Ok(uberblock) = Uberblock::from_bytes(&self.read(256 + i * 2, 2)) {
                let newest = match newest_uberblock {
                    Some(previous) => {
                        if uberblock.txg > previous.txg {
                            // Found a newer uberblock
                            true
                        } else {
                            false
                        }
                    }
                    // No uberblock yet, so first one we find is the newest
                    None => true,
                };

                if newest {
                    newest_uberblock = Some(uberblock);
                }
            }
        }

        match newest_uberblock {
            Some(uberblock) => Ok(uberblock),
            None => Err("Failed to find valid uberblock".to_string()),
        }
    }
}
