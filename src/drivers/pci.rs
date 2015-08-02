use core::clone::Clone;
use core::result::Result;

use common::debug::*;
use common::pci::*;
use common::string::*;
use common::url::*;

use network::intel8254x::*;
use network::rtl8139::*;

use programs::session::*;

use usb::xhci::*;

struct PCI;

impl SessionScheme for PCI {
    fn on_url(&mut self, session: &Session, url: &URL){
        if url.scheme == "pci".to_string() {
            d("PCI URL ");
            url.d();

            let mut bus = -1;
            let mut slot = -1;
            let mut func = -1;
            let mut reg = String::new();

            for i in 0..url.path.len() {
                match url.path.get(i){
                    Result::Ok(part) => match i {
                        0 => {
                            bus = part.to_num() as isize;
                        },
                        1 => {
                            slot = part.to_num() as isize;
                        },
                        2 => {
                            func = part.to_num() as isize;
                        },
                        3 => {
                            reg = part.clone();
                        },
                        _ => ()
                    },
                    Result::Err(_) => ()
                }
            }

            let ret;
            if bus >= 0 {
                if slot >= 0 {
                    if func >= 0 {
                        if reg.len() > 0 {
                            if reg == "class".to_string() {
                                unsafe {
                                    ret = String::from_num_radix((pci_read(bus as usize, slot as usize, func as usize, 8) >> 24) & 0xFF, 16);
                                }
                            }else{
                                ret = "Unknown reg ".to_string() + reg.clone();
                            }
                        }else{
                            ret = String::from_num(256);
                        }
                    }else{
                        ret = String::from_num(8);
                    }
                }else{
                    ret = String::from_num(32);
                }
            }else{
                ret = String::from_num(256);
            }
            ret.d();
            dl();
        }
    }
}

pub unsafe fn pci_device(session: &mut Session, bus: usize, slot: usize, func: usize, class_id: usize, subclass_id: usize, interface_id: usize, vendor_code: usize, device_code: usize){
    if class_id == 0x01 && subclass_id == 0x01{
        let base = pci_read(bus, slot, func, 0x20);

        d("IDE Controller on ");
        dh(base & 0xFFFFFFF0);
        dl();
    }else if class_id == 0x0C && subclass_id == 0x03{
        if interface_id == 0x30{
            let base = pci_read(bus, slot, func, 0x10);

            let session_device = box XHCI {
                bus: bus,
                slot: slot,
                func: func,
                base: base & 0xFFFFFFF0,
                memory_mapped: base & 1 == 0,
                irq: pci_read(bus, slot, 0, 0x3C) as u8 & 0xF
            };
            session_device.init();
            session.devices.push(session_device);
        }else if interface_id == 0x20{
            let base = pci_read(bus, slot, func, 0x10);

            d("EHCI Controller on ");
            dh(base & 0xFFFFFFF0);
            dl();
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
                    let session_device = box RTL8139 {
                        bus: bus,
                        slot: slot,
                        func: func,
                        base: base & 0xFFFFFFF0,
                        memory_mapped: base & 1 == 0,
                        irq: pci_read(bus, slot, 0, 0x3C) as u8 & 0xF
                    };
                    session_device.init();
                    session.devices.push(session_device);
                },
                _ => ()
            },
            0x8086 => match device_code{ // INTEL
                0x100E => {
                    let base = pci_read(bus, slot, 0, 0x10);
                    let session_device = box Intel8254x {
                        bus: bus,
                        slot: slot,
                        func: func,
                        base: base & 0xFFFFFFF0,
                        memory_mapped: base & 1 == 0,
                        irq: pci_read(bus, slot, 0, 0x3C) as u8 & 0xF
                    };
                    session_device.init();
                    session.devices.push(session_device);
                },
                _ => ()
            },
            _ => ()
        }
    }
}

pub unsafe fn pci_init(session: &mut Session){
    d("Add PCI Scheme\n");
    session.schemes.push(box PCI);

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
                    dl();

                    for i in 0..6 {
                        d("    ");
                        dd(i);
                        d(": ");
                        dh(pci_read(bus, slot, func, i*4 + 0x10));
                        dl();
                    }

                    pci_device(session, bus, slot, func, (class_id >> 24) & 0xFF, (class_id >> 16) & 0xFF, (class_id >> 8) & 0xFF, data & 0xFFFF, (data >> 16) & 0xFFFF);

                    let url = URL::from_string("pci:///".to_string() + bus + "/" + slot + "/" + func + "/class");
                    session.on_url(&url);

                    dl();
                }
            }
        }
    }
}
