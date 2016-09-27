use io::Io;

use syscall::error::Result;

use self::dma::Dma;
use self::hba::{HbaMem, HbaCmdTable, HbaCmdHeader, HbaPort, HbaPortType};

pub mod dma;
pub mod fis;
pub mod hba;

pub struct Ahci;

impl Ahci {
    pub fn disks(base: usize, irq: u8) -> Vec<AhciDisk> {
        println!(" + AHCI on: {:X} IRQ: {}", base as usize, irq);

        let pi = unsafe { &mut *(base as *mut HbaMem) }.pi.read();
        let ret: Vec<AhciDisk> = (0..32)
              .filter(|&i| pi & 1 << i as i32 == 1 << i as i32)
              .filter_map(|i| {
                  let port = &mut unsafe { &mut *(base as *mut HbaMem) }.ports[i];
                  let port_type = port.probe();
                  println!("{}: {:?}", i, port_type);
                  match port_type {
                      HbaPortType::SATA => {
                          match AhciDisk::new(port) {
                              Ok(disk) => Some(disk),
                              Err(err) => {
                                  println!("{}: {}", i, err);
                                  None
                              }
                          }
                      }
                      _ => None,
                  }
              })
              .collect();

        ret
    }
}

pub struct AhciDisk {
    port: &'static mut HbaPort,
    size: u64,
    clb: Dma<[HbaCmdHeader; 32]>,
    ctbas: [Dma<HbaCmdTable>; 32],
    fb: Dma<[u8; 256]>
}

impl AhciDisk {
    fn new(port: &'static mut HbaPort) -> Result<Self> {
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

        port.init(&mut clb, &mut ctbas, &mut fb);

        let size = unsafe { port.identify(&mut clb, &mut ctbas).unwrap_or(0) };

        Ok(AhciDisk {
            port: port,
            size: size,
            clb: clb,
            ctbas: ctbas,
            fb: fb
        })
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn read(&mut self, block: u64, buffer: &mut [u8]) -> Result<usize> {
        self.port.ata_dma(block, buffer.len() / 512, buffer.as_ptr() as usize, false)
    }

    fn write(&mut self, block: u64, buffer: &[u8]) -> Result<usize> {
        self.port.ata_dma(block, buffer.len() / 512, buffer.as_ptr() as usize, true)
    }
}
