use io::Io;

use syscall::error::Result;

use self::hba::{HbaMem, HbaPort, HbaPortType};

pub mod fis;
pub mod hba;

pub struct Ahci;

impl Ahci {
    pub fn disks(base: usize, irq: u8) -> Vec<AhciDisk> {
        println!(" + AHCI on: {:X} IRQ: {:X}", base as usize, irq);

        let pi = unsafe { &mut *(base as *mut HbaMem) }.pi.read();
        let ret: Vec<AhciDisk> = (0..32)
              .filter(|&i| pi & 1 << i as i32 == 1 << i as i32)
              .filter_map(|i| {
                  let mut disk = AhciDisk::new(base, i, irq);
                  let port_type = disk.port.probe();
                  println!("{}: {:?}", i, port_type);
                  match port_type {
                      HbaPortType::SATA => {
                          /*
                          disk.port.init();
                          if let Some(size) = unsafe { disk.port.identify(i) } {
                              disk.size = size;
                              Some(disk)
                          } else {
                              None
                          }
                          */
                          None
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
    port_index: usize,
    irq: u8,
    size: u64,
}

impl AhciDisk {
    fn new(base: usize, port_index: usize, irq: u8) -> Self {
        AhciDisk {
            port: &mut unsafe { &mut *(base as *mut HbaMem) }.ports[port_index],
            port_index: port_index,
            irq: irq,
            size: 0
        }
    }

    fn name(&self) -> String {
        format!("AHCI Port {}", self.port_index)
    }

    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            //debugln!("AHCI IRQ");
        }
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
