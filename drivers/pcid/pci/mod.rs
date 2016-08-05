extern crate io;

use self::io::{Io, Pio};

use std::cell::RefCell;

pub use self::bar::PciBar;
pub use self::bus::{PciBus, PciBusIter};
pub use self::class::PciClass;
pub use self::dev::{PciDev, PciDevIter};
pub use self::func::PciFunc;
pub use self::header::PciHeader;

mod bar;
mod bus;
mod class;
mod dev;
mod func;
mod header;

pub struct Pci {
    addr: RefCell<Pio<u32>>,
    data: Pio<u32>
}

impl Pci {
    pub fn new() -> Self {
        Pci {
            addr: RefCell::new(Pio::new(0xCF8)),
            data: Pio::new(0xCFC)
        }
    }

    pub fn buses<'pci>(&'pci self) -> PciIter<'pci> {
        PciIter::new(self)
    }

    pub unsafe fn read(&self, bus: u8, dev: u8, func: u8, offset: u8) -> u32 {
        let address = 0x80000000 | ((bus as u32) << 16) | ((dev as u32) << 11) | ((func as u32) << 8) | ((offset as u32) & 0xFC);
        self.addr.borrow_mut().write(address);
        self.data.read()
    }
}

pub struct PciIter<'pci> {
    pci: &'pci Pci,
    num: u32
}

impl<'pci> PciIter<'pci> {
    pub fn new(pci: &'pci Pci) -> Self {
        PciIter {
            pci: pci,
            num: 0
        }
    }
}

impl<'pci> Iterator for PciIter<'pci> {
    type Item = PciBus<'pci>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.num < 256 {
            let bus = PciBus {
                pci: self.pci,
                num: self.num as u8
            };
            self.num += 1;
            Some(bus)
        } else {
            None
        }
    }
}
