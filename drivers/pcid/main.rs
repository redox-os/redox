extern crate system;

use system::syscall::sys_iopl;

use std::thread;

use pci::{Pci, PciClass};

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
                }
            }
        }
    }
}

fn main() {
    thread::spawn(|| {
        unsafe { sys_iopl(3).unwrap() };

        enumerate_pci();

        thread::sleep_ms(10000);
    });
}
