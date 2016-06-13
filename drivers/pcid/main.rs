extern crate system;

use system::syscall::sys_iopl;

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
                }
            }
        }
    }
}

fn main() {
    unsafe { sys_iopl(3).unwrap() };

    enumerate_pci();
}
