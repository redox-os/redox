use audio::ac97::AC97;
use audio::intelhda::IntelHDA;

use collections::vec::Vec;

use common::debug;
use common::queue::Queue;

use drivers::pciconfig::PciConfig;

use network::intel8254x::Intel8254x;
use network::rtl8139::Rtl8139;

use programs::session::Session;

use schemes::file::FileScheme;

use usb::ehci::Ehci;
use usb::uhci::Uhci;
use usb::xhci::Xhci;

/// PCI device
pub unsafe fn pci_device(session: &mut Session,
                         mut pci: PciConfig,
                         class_id: u32,
                         subclass_id: u32,
                         interface_id: u32,
                         vendor_code: u32,
                         device_code: u32) {
    if class_id == 0x01 && subclass_id == 0x01 {
        if let Some(module) = FileScheme::new(pci) {
            session.items.push(module);
        }
    } else if class_id == 0x0C && subclass_id == 0x03 {
        if interface_id == 0x30 {
            let base = pci.read(0x10) as usize;

            let module = box Xhci {
                pci: pci,
                base: base & 0xFFFFFFF0,
                memory_mapped: base & 1 == 0,
                irq: pci.read(0x3C) as u8 & 0xF,
            };
            module.init();
            session.items.push(module);
        } else if interface_id == 0x20 {
            let base = pci.read(0x10) as usize;

            let mut module = box Ehci {
                pci: pci,
                base: base & 0xFFFFFFF0,
                memory_mapped: base & 1 == 0,
                irq: pci.read(0x3C) as u8 & 0xF,
            };
            module.init();
            session.items.push(module);
        } else if interface_id == 0x10 {
            let base = pci.read(0x10) as usize;

            debug!("OHCI Controller on {}\n", base & 0xFFFFFFF0);
        } else if interface_id == 0x00 {
            session.items.push(Uhci::new(pci));
        } else {
            debug!("Unknown USB interface version\n");
        }
    } else {
        match vendor_code {
            0x10EC => match device_code { // REALTEK
                0x8139 => {
                    session.items.push(Rtl8139::new(pci));
                }
                _ => (),
            },
            0x8086 => match device_code { // INTEL
                0x100E => {
                    let base = pci.read(0x10) as usize;
                    let mut module = box Intel8254x {
                        pci: pci,
                        base: base & 0xFFFFFFF0,
                        memory_mapped: base & 1 == 0,
                        irq: pci.read(0x3C) as u8 & 0xF,
                        resources: Vec::new(),
                        inbound: Queue::new(),
                        outbound: Queue::new(),
                    };
                    module.init();
                    session.items.push(module);
                }
                0x2415 => session.items.push(AC97::new(pci)),
                0x24C5 => session.items.push(AC97::new(pci)),
                0x2668 => {
                    let base = pci.read(0x10) as usize;
                    let mut module = box IntelHDA {
                        pci: pci,
                        base: base & 0xFFFFFFF0,
                        memory_mapped: base & 1 == 0,
                        irq: pci.read(0x3C) as u8 & 0xF,
                    };
                    module.init();
                    session.items.push(module);
                }
                _ => (),
            },
            _ => (),
        }
    }
}

/// Initialize PCI session
pub unsafe fn pci_init(session: &mut Session) {
    for bus in 0..256 {
        for slot in 0..32 {
            for func in 0..8 {
                let mut pci = PciConfig::new(bus as u8, slot as u8, func as u8);
                let id = pci.read(0);

                if (id & 0xFFFF) != 0xFFFF {
                    let class_id = pci.read(8);

                    debug!(" * PCI {}, {}, {}: ID {:X} CL {:X}",
                           bus,
                           slot,
                           func,
                           id,
                           class_id);

                    for i in 0..6 {
                        let bar = pci.read(i * 4 + 0x10);
                        if bar > 0 {
                            debug!(" BAR{}: {:X}", i, bar);

                            pci.write(i * 4 + 0x10, 0xFFFFFFFF);
                            let size = (0xFFFFFFFF - (pci.read(i * 4 + 0x10) & 0xFFFFFFF0)) + 1;
                            pci.write(i * 4 + 0x10, bar);

                            if size > 0 {
                                debug!(" {}", size);
                            }
                        }
                    }

                    debug::dl();

                    pci_device(session,
                               pci,
                               (class_id >> 24) & 0xFF,
                               (class_id >> 16) & 0xFF,
                               (class_id >> 8) & 0xFF,
                               id & 0xFFFF,
                               (id >> 16) & 0xFFFF);
                }
            }
        }
    }
}
