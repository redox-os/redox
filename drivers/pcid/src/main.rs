#![feature(asm)]

extern crate syscall;

use syscall::iopl;

use pci::{Pci, PciBar, PciClass};

mod pci;

fn enumerate_pci() {
    println!("PCI BS/DV/FN VEND:DEVI CL.SC.IN.RV");

    let pci = Pci::new();
    for bus in pci.buses() {
        for dev in bus.devs() {
            for func in dev.funcs() {
                if let Some(header) = func.header() {
                    println!("PCI {:>02X}/{:>02X}/{:>02X} {:>04X}:{:>04X} {:>02X}.{:>02X}.{:>02X}.{:>02X} {:?}",
                            bus.num, dev.num, func.num,
                            header.vendor_id, header.device_id,
                            header.class, header.subclass, header.interface, header.revision,
                            PciClass::from(header.class));

                    for i in 0..header.bars.len() {
                        match PciBar::from(header.bars[i]) {
                            PciBar::None => (),
                            PciBar::Memory(address) => println!("    BAR {} {:>08X}", i, address),
                            PciBar::Port(address) => println!("    BAR {} {:>04X}", i, address)
                        }
                    }

                    match PciClass::from(header.class) {
                        PciClass::Storage => match header.subclass {
                            0x01 => {
                                println!("    + IDE");
                            },
                            0x06 => {
                                println!("    + SATA");
                            },
                            _ => ()
                        },
                        PciClass::SerialBus => match header.subclass {
                            0x03 => match header.interface {
                                0x00 => {
                                    println!("    + UHCI");
                                },
                                0x10 => {
                                    println!("    + OHCI");
                                },
                                0x20 => {
                                    println!("    + EHCI");
                                },
                                0x30 => {
                                    println!("    + XHCI");
                                },
                                _ => ()
                            },
                            _ => ()
                        },
                        _ => ()
                    }
                }
            }
        }
    }
}

fn main() {
    unsafe { iopl(3).unwrap() };

    enumerate_pci();
}
