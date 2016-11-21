#![feature(asm)]

extern crate rustc_serialize;
extern crate syscall;
extern crate toml;

use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use syscall::iopl;

use config::Config;
use pci::{Pci, PciBar, PciClass};

mod config;
mod pci;

fn main() {
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

    unsafe { iopl(3).unwrap() };

    print!("PCI BS/DV/FN VEND:DEVI CL.SC.IN.RV\n");

    let pci = Pci::new();
    for bus in pci.buses() {
        for dev in bus.devs() {
            for func in dev.funcs() {
                if let Some(header) = func.header() {
                    let pci_class = PciClass::from(header.class);

                    let mut string = format!("PCI {:>02X}/{:>02X}/{:>02X} {:>04X}:{:>04X} {:>02X}.{:>02X}.{:>02X}.{:>02X} {:?}",
                            bus.num, dev.num, func.num,
                            header.vendor_id, header.device_id,
                            header.class, header.subclass, header.interface, header.revision,
                            pci_class);

                    match pci_class {
                        PciClass::Storage => match header.subclass {
                            0x01 => {
                                string.push_str(" IDE");
                            },
                            0x06 => {
                                string.push_str(" SATA");
                            },
                            _ => ()
                        },
                        PciClass::SerialBus => match header.subclass {
                            0x03 => match header.interface {
                                0x00 => {
                                    string.push_str(" UHCI");
                                },
                                0x10 => {
                                    string.push_str(" OHCI");
                                },
                                0x20 => {
                                    string.push_str(" EHCI");
                                },
                                0x30 => {
                                    string.push_str(" XHCI");
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
                            PciBar::Memory(address) => string.push_str(&format!(" {}={:>08X}", i, address)),
                            PciBar::Port(address) => string.push_str(&format!(" {}={:>04X}", i, address))
                        }
                    }

                    string.push('\n');

                    print!("{}", string);

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
                            // Enable bus mastering
                            unsafe {
                                let cmd = pci.read(bus.num, dev.num, func.num, 0x04);
                                pci.write(bus.num, dev.num, func.num, 0x04, cmd | 4);
                            }

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
                                        "$BUS" => format!("{:>02X}", bus.num),
                                        "$DEV" => format!("{:>02X}", dev.num),
                                        "$FUNC" => format!("{:>02X}", func.num),
                                        "$NAME" => format!("pci-{:>02X}.{:>02X}.{:>02X}", bus.num, dev.num, func.num),
                                        "$BAR0" => bar_arg(0),
                                        "$BAR1" => bar_arg(1),
                                        "$BAR2" => bar_arg(2),
                                        "$BAR3" => bar_arg(3),
                                        "$BAR4" => bar_arg(4),
                                        "$BAR5" => bar_arg(5),
                                        "$IRQ" => format!("{}", header.interrupt_line),
                                        _ => arg.clone()
                                    };
                                    command.arg(&arg);
                                }

                                println!("PCID SPAWN {:?}", command);

                                match command.spawn() {
                                    Ok(mut child) => match child.wait() {
                                        Ok(_status) => (), //println!("pcid: waited for {}: {:?}", line, status.code()),
                                        Err(err) => println!("pcid: failed to wait for {:?}: {}", command, err)
                                    },
                                    Err(err) => println!("pcid: failed to execute {:?}: {}", command, err)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
