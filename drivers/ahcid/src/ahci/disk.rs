use std::ptr;

use dma::Dma;
use syscall::error::Result;

use super::hba::{HbaPort, HbaCmdTable, HbaCmdHeader};

pub struct Disk {
    id: usize,
    port: &'static mut HbaPort,
    size: u64,
    clb: Dma<[HbaCmdHeader; 32]>,
    ctbas: [Dma<HbaCmdTable>; 32],
    _fb: Dma<[u8; 256]>,
    buf: Dma<[u8; 256 * 512]>
}

impl Disk {
    pub fn new(id: usize, port: &'static mut HbaPort) -> Result<Self> {
        let mut clb = Dma::zeroed()?;
        let mut ctbas = [
            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
        ];
        let mut fb = Dma::zeroed()?;
        let buf = Dma::zeroed()?;

        port.init(&mut clb, &mut ctbas, &mut fb);

        let size = unsafe { port.identify(&mut clb, &mut ctbas).unwrap_or(0) };

        Ok(Disk {
            id: id,
            port: port,
            size: size,
            clb: clb,
            ctbas: ctbas,
            _fb: fb,
            buf: buf
        })
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn read(&mut self, block: u64, buffer: &mut [u8]) -> Result<usize> {
        let sectors = buffer.len()/512;

        let mut sector: usize = 0;
        while sectors - sector >= 255 {
            if let Err(err) = self.port.ata_dma(block + sector as u64, 255, false, &mut self.clb, &mut self.ctbas, &mut self.buf) {
                return Err(err);
            }

            unsafe { ptr::copy(self.buf.as_ptr(), buffer.as_mut_ptr().offset(sector as isize * 512), 255 * 512); }

            sector += 255;
        }
        if sector < sectors {
            if let Err(err) = self.port.ata_dma(block + sector as u64, sectors - sector, false, &mut self.clb, &mut self.ctbas, &mut self.buf) {
                return Err(err);
            }

            unsafe { ptr::copy(self.buf.as_ptr(), buffer.as_mut_ptr().offset(sector as isize * 512), (sectors - sector) * 512); }

            sector += sectors - sector;
        }

        Ok(sector * 512)
    }

    pub fn write(&mut self, block: u64, buffer: &[u8]) -> Result<usize> {
        let sectors = buffer.len()/512;

        let mut sector: usize = 0;
        while sectors - sector >= 255 {
            unsafe { ptr::copy(buffer.as_ptr().offset(sector as isize * 512), self.buf.as_mut_ptr(), 255 * 512); }

            if let Err(err) = self.port.ata_dma(block + sector as u64, 255, true, &mut self.clb, &mut self.ctbas, &mut self.buf) {
                return Err(err);
            }

            sector += 255;
        }
        if sector < sectors {
            unsafe { ptr::copy(buffer.as_ptr().offset(sector as isize * 512), self.buf.as_mut_ptr(), (sectors - sector) * 512); }

            if let Err(err) = self.port.ata_dma(block + sector as u64, sectors - sector, true, &mut self.clb, &mut self.ctbas, &mut self.buf) {
                return Err(err);
            }

            sector += sectors - sector;
        }

        Ok(sector * 512)
    }
}
