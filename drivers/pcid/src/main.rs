#![feature(asm)]

extern crate syscall;

use std::fs::File;
use std::io::{Read, Write};
use std::thread;
use syscall::{iopl, Packet};

use pci::{Pci, PciBar, PciClass};

mod pci;

fn enumerate_pci() {
    println!("PCI BS/DV/FN VEND:DEVI CL.SC.IN.RV");

    let pci = Pci::new();
    for bus in pci.buses() {
        for dev in bus.devs() {
            for func in dev.funcs() {
                if let Some(header) = func.header() {
                    print!("PCI {:>02X}/{:>02X}/{:>02X} {:>04X}:{:>04X} {:>02X}.{:>02X}.{:>02X}.{:>02X}",
                            bus.num, dev.num, func.num,
                            header.vendor_id, header.device_id,
                            header.class, header.subclass, header.interface, header.revision);

                    let pci_class = PciClass::from(header.class);
                    print!(" {:?}", pci_class);
                    match pci_class {
                        PciClass::Storage => match header.subclass {
                            0x01 => {
                                print!(" IDE");
                            },
                            0x06 => {
                                print!(" SATA");
                            },
                            _ => ()
                        },
                        PciClass::SerialBus => match header.subclass {
                            0x03 => match header.interface {
                                0x00 => {
                                    print!(" UHCI");
                                },
                                0x10 => {
                                    print!(" OHCI");
                                },
                                0x20 => {
                                    print!(" EHCI");
                                },
                                0x30 => {
                                    print!(" XHCI");
                                },
                                _ => ()
                            },
                            _ => ()
                        },
                        _ => ()
                    }

                    for i in 0..header.bars.len() {
                        match PciBar::from(header.bars[i]) {
                            PciBar::None => (),
                            PciBar::Memory(address) => print!(" {}={:>08X}", i, address),
                            PciBar::Port(address) => print!(" {}={:>04X}", i, address)
                        }
                    }

                    print!("\n");
                }
            }
        }
    }
}

fn main() {
    thread::spawn(||{
        unsafe { iopl(3).unwrap() };

        enumerate_pci();

        let mut scheme = File::create(":pci").expect("pcid: failed to create pci scheme");
        loop {
            let mut packet = Packet::default();
            scheme.read(&mut packet).expect("pcid: failed to read events from pci scheme");

            println!("{:?}", packet);

            packet.a = 0;

            scheme.write(&packet).expect("pcid: failed to write responses to pci scheme");
        }
    });
}
