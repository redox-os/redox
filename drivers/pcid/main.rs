extern crate system;

use system::syscall::sys_iopl;

use std::thread;

use pci::{Pci, PciClass};

mod pci;

fn main() {
    thread::spawn(|| {
        unsafe { sys_iopl(3).unwrap() };

        let pci = Pci::new();
        for bus in pci.buses() {
            for dev in bus.devs() {
                for func in dev.funcs() {
                    if let Some(header) = func.header() {
                        println!("PCI {:X} {:X} {:X}: {:?} {:#?}", bus.num, dev.num, func.num, PciClass::from(header.class), header);
                    }
                }
            }
        }

        thread::sleep_ms(10000);
    });
}
