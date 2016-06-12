extern crate io;
extern crate system;

use io::{Io, Pio};

use system::syscall::sys_iopl;

use std::cell::RefCell;
use std::thread;

#[derive(Debug)]
enum PciClass {
    Legacy,
    Storage,
    Network,
    Display,
    Multimedia,
    Memory,
    Bridge,
    SimpleComms,
    Peripheral,
    Input,
    Docking,
    Processor,
    SerialBus,
    Wireless,
    IntelligentIo,
    SatelliteComms,
    Cryptography,
    SignalProc,
    Reserved(u8),
    Unknown
}

impl From<u8> for PciClass {
    fn from(class: u8) -> PciClass {
        match class {
            0x00 => PciClass::Legacy,
            0x01 => PciClass::Storage,
            0x02 => PciClass::Network,
            0x03 => PciClass::Display,
            0x04 => PciClass::Multimedia,
            0x05 => PciClass::Memory,
            0x06 => PciClass::Bridge,
            0x07 => PciClass::SimpleComms,
            0x08 => PciClass::Peripheral,
            0x09 => PciClass::Input,
            0x0A => PciClass::Docking,
            0x0B => PciClass::Processor,
            0x0C => PciClass::SerialBus,
            0x0D => PciClass::Wireless,
            0x0E => PciClass::IntelligentIo,
            0x0F => PciClass::SatelliteComms,
            0x10 => PciClass::Cryptography,
            0x11 => PciClass::SignalProc,
            0xFF => PciClass::Unknown,
            reserved => PciClass::Reserved(reserved)
        }
    }
}

struct Pci {
    addr: RefCell<Pio<u32>>,
    data: Pio<u32>
}

impl Pci {
    fn new() -> Pci {
        Pci {
            addr: RefCell::new(Pio::new(0xCF8)),
            data: Pio::new(0xCFC)
        }
    }

    fn read(&self, bus: u8, dev: u8, func: u8, offset: u8) -> u32 {
        let address = 0x80000000 | ((bus as u32) << 16) | ((dev as u32) << 11) | ((func as u32) << 8) | ((offset as u32) & 0xFC);
        self.addr.borrow_mut().write(address);
        self.data.read()
    }
}

struct PciBus<'pci> {
    pci: &'pci mut Pci,
    bus: u8
}

impl<'pci> PciBus<'pci> {
    fn read(&self, slot: u8, func: u8, offset: u8) -> u32 {
        self.pci.read(self.bus, slot, func, offset)
    }
}

struct PciDev<'pci> {
    bus: &'pci mut PciBus<'pci>,
    dev: u8
}

impl<'pci> PciDev<'pci> {
    fn read(&self, func: u8, offset: u8) -> u32 {
        self.bus.read(self.dev, func, offset)
    }
}

struct PciFunc<'pci> {
    dev: &'pci mut PciDev<'pci>,
    func: u8
}

impl<'pci> PciFunc<'pci> {
    fn read(&self, offset: u8) -> u32 {
        self.dev.read(self.func, offset)
    }
}

fn main() {
    thread::spawn(|| {
        unsafe { sys_iopl(3).unwrap() };

        let pci = Pci::new();
        for bus in 0..256 {
            for dev in 0..32 {
                for func in 0..8 {
                    let id = pci.read(bus as u8, dev as u8, func as u8, 0x00);
                    if id != 0xFFFFFFFF {
                        let class = pci.read(bus as u8, dev as u8, func as u8, 0x08);
                        println!("PCI {:X} {:X} {:X}: {:X} {:X} {:?}", bus, dev, func, id, class, PciClass::from((class >> 24) as u8));
                    }
                }
            }
        }

        thread::sleep_ms(10000);
    });
}
