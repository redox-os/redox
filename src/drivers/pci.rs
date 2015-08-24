use common::debug::*;
use common::pci::*;
use common::vec::*;

use network::intel8254x::*;
use network::rtl8139::*;

use programs::session::*;

use schemes::ide::*;

use usb::ehci::*;
use usb::xhci::*;

pub unsafe fn pci_device(session: &mut Session, bus: usize, slot: usize, func: usize, class_id: usize, subclass_id: usize, interface_id: usize, vendor_code: usize, device_code: usize){
    if class_id == 0x01 && subclass_id == 0x01{
        let base = pci_read(bus, slot, func, 0x20);

        let module = box IDE {
            bus: bus,
            slot: slot,
            func: func,
            base: base & 0xFFFFFFF0,
            memory_mapped: base & 1 == 0,
            requests: Vec::new()
        };
        module.init();
        session.modules.push(module);
    }else if class_id == 0x0C && subclass_id == 0x03{
        if interface_id == 0x30{
            let base = pci_read(bus, slot, func, 0x10);

            let module = box XHCI {
                bus: bus,
                slot: slot,
                func: func,
                base: base & 0xFFFFFFF0,
                memory_mapped: base & 1 == 0,
                irq: pci_read(bus, slot, func, 0x3C) as u8 & 0xF
            };
            module.init();
            session.modules.push(module);
        }else if interface_id == 0x20{
            let base = pci_read(bus, slot, func, 0x10);

            let module = box EHCI {
                bus: bus,
                slot: slot,
                func: func,
                base: base & 0xFFFFFFF0,
                memory_mapped: base & 1 == 0,
                irq: pci_read(bus, slot, func, 0x3C) as u8 & 0xF
            };
            module.init();
            session.modules.push(module);
        }else if interface_id == 0x10{
            let base = pci_read(bus, slot, func, 0x10);

            d("OHCI Controller on ");
            dh(base & 0xFFFFFFF0);
            dl();
        }else if interface_id == 0x00{
            let base = pci_read(bus, slot, func, 0x20);

            d("UHCI Controller on ");
            dh(base & 0xFFFFFFF0);
            dl();
        }else{
            d("Unknown USB interface version\n");
        }
    }else{
        match vendor_code {
            0x10EC => match device_code{ // REALTEK
                0x8139 => {
                    let base = pci_read(bus, slot, func, 0x10);
                    let module = box RTL8139 {
                        bus: bus,
                        slot: slot,
                        func: func,
                        base: base & 0xFFFFFFF0,
                        memory_mapped: base & 1 == 0,
                        irq: pci_read(bus, slot, func, 0x3C) as u8 & 0xF
                    };
                    module.init();
                    session.modules.push(module);
                },
                _ => ()
            },
            0x8086 => match device_code{ // INTEL
                0x100E => {
                    let base = pci_read(bus, slot, 0, 0x10);
                    let module = box Intel8254x {
                        bus: bus,
                        slot: slot,
                        func: func,
                        base: base & 0xFFFFFFF0,
                        memory_mapped: base & 1 == 0,
                        irq: pci_read(bus, slot, func, 0x3C) as u8 & 0xF
                    };
                    module.init();
                    session.modules.push(module);
                },
                _ => ()
            },
            _ => ()
        }
    }
}

pub unsafe fn pci_init(session: &mut Session){
    for bus in 0..256 {
        for slot in 0..32 {
            for func in 0..8 {
                let data = pci_read(bus, slot, func, 0);

                if (data & 0xFFFF) != 0xFFFF {
                    let class_id = pci_read(bus, slot, func, 8);

                    d("Bus ");
                    dd(bus);
                    d(" Slot ");
                    dd(slot);
                    d(" Function ");
                    dd(func);
                    d(": ");
                    dh(data);
                    d(", ");
                    dh(class_id);

                    for i in 0..6 {
                        let bar = pci_read(bus, slot, func, i*4 + 0x10);
                        if bar > 0 {
                            d(" BAR");
                            dd(i);
                            d(": ");
                            dh(bar);
                        }
                    }

                    dl();

                    pci_device(session, bus, slot, func, (class_id >> 24) & 0xFF, (class_id >> 16) & 0xFF, (class_id >> 8) & 0xFF, data & 0xFFFF, (data >> 16) & 0xFFFF);
                }
            }
        }
    }
}
