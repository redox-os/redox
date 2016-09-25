#![feature(asm)]

extern crate rustc_serialize;
extern crate syscall;
extern crate toml;

use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::thread;
use syscall::iopl;

use config::Config;
use pci::{Pci, PciBar, PciClass};

mod config;
mod pci;

fn main() {
    thread::spawn(|| {
        let mut config = Config::default();

        let mut args = env::args().skip(1);
        if let Some(config_path) = args.next() {
            if let Ok(mut config_file) = File::open(&config_path) {
                let mut config_data = String::new();
                if let Ok(_) = config_file.read_to_string(&mut config_data) {
                    config = toml::decode_str(&config_data).unwrap_or(Config::default());
                }
            }
        }

        println!("{:?}", config);

        unsafe { iopl(3).unwrap() };

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

                        for driver in config.drivers.iter() {
                            if let Some(class) = driver.class {
                                if class != header.class { continue; }
                            }

                            if let Some(subclass) = driver.subclass {
                                if subclass != header.subclass { continue; }
                            }

                            if let Some(vendor) = driver.vendor {
                                if vendor != header.vendor_id { continue; }
                            }

                            if let Some(device) = driver.device {
                                if device != header.device_id { continue; }
                            }

                            if let Some(ref args) = driver.command {
                                let mut args = args.iter();
                                if let Some(program) = args.next() {
                                    let mut command = Command::new(program);
                                    for arg in args {
                                        let bar_arg = |i| -> String {
                                            match PciBar::from(header.bars[i]) {
                                                PciBar::None => String::new(),
                                                PciBar::Memory(address) => format!("{:>08X}", address),
                                                PciBar::Port(address) => format!("{:>04X}", address)
                                            }
                                        };
                                        let arg = match arg.as_str() {
                                            "$0" => bar_arg(0),
                                            "$1" => bar_arg(1),
                                            "$2" => bar_arg(2),
                                            "$3" => bar_arg(3),
                                            "$4" => bar_arg(4),
                                            "$5" => bar_arg(5),
                                            _ => arg.clone()
                                        };
                                        command.arg(&arg);
                                    }
                                    println!("{:?}", command);
                                }
                            }

                            println!("Driver: {:?}", driver);
                        }
                    }
                }
            }
        }
    });
}
