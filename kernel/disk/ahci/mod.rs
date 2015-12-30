use alloc::boxed::Box;

use common::memory;

use drivers::pciconfig::PciConfig;

use schemes::KScheme;

use self::hba::{HbaMem, HbaPortType};

pub mod fis;
pub mod hba;

pub struct Ahci {
    pci: PciConfig,
    mem: *mut HbaMem,
    irq: u8,
}

impl Ahci {
    pub fn new(mut pci: PciConfig) -> Box<Self> {
        let base = unsafe { (pci.read(0x24) & 0xFFFFFFF0) as usize };
        let irq = unsafe { (pci.read(0x3C) & 0xF) as u8 };

        let mut module = box Ahci {
            pci: pci,
            mem: base as *mut HbaMem,
            irq: irq,
        };

        module.init();

        module
    }

    fn init(&mut self) {
        debugln!("AHCI on: {:X} IRQ: {:X}", self.mem as usize, self.irq);

        let mem = unsafe { &mut * self.mem };

        for i in 0..32 {
            if mem.pi.read() & 1 << i == 1 << i {
                let port = &mut mem.ports[i];
                let port_type = port.probe();
                debugln!("Port {}: {:?}", i, port_type);
                match port_type {
                    HbaPortType::SATA => {
                        port.init();

                        let buffer = unsafe { memory::alloc(1024) };
                        if buffer as usize > 0 {
                            if port.read(0, buffer, 1024) {
                                for i in 0..1024 {
                                    debug!("{:02X} ", unsafe { *(buffer as *const u8).offset(i) });
                                }
                                debugln!("");
                            }
                            unsafe { memory::unalloc(buffer) };
                        }
                    },
                    _ => ()
                }
            }
        }
    }
}

impl KScheme for Ahci {
    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            debugln!("AHCI IRQ");
        }
    }

    fn on_poll(&mut self) {
    }
}
