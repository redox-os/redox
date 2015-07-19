use common::debug::*;
use common::pci::*;

use network::intel8254x::*;
use network::rtl8139::*;

use programs::session::*;

pub unsafe fn pci_device(session: &mut Session, bus: usize, slot: usize, vendor_code: usize, device_code: usize){
    match(vendor_code){
        0x10EC => match(device_code){ // REALTEK
            0x8139 => {
                let base = pci_read(bus, slot, 0, 0x10);
                let mut session_device = box RTL8139 {
                    bus: bus,
                    slot: slot,
                    base: base & (0xFFFFFFFF - 1),
                    memory_mapped: base & 1 == 0,
                    irq: pci_read(bus, slot, 0, 0x3C) as u8 & 0xF
                };
                session_device.init();
                session.devices.push(session_device);
            },
            _ => ()
        },
        0x8086 => match(device_code){ // INTEL
            0x100E => {
                let base = pci_read(bus, slot, 0, 0x10);
                let mut session_device = box Intel8254x {
                    bus: bus,
                    slot: slot,
                    base: base & (0xFFFFFFFF - 1),
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

pub unsafe fn pci_init(session: &mut Session){
    for bus in 0..256 {
        for slot in 0..32 {
            let data = pci_read(bus, slot, 0, 0);

            if (data & 0xFFFF) != 0xFFFF {
                d("Bus ");
                dd(bus);
                d(" Slot ");
                dd(slot);
                d(": ");
                dh(data);
                d(", ");
                dh(pci_read(bus, slot, 0, 8));
                dl();

                for i in 0..6 {
                    d("    ");
                    dd(i);
                    d(": ");
                    dh(pci_read(bus, slot, 0, i*4 + 0x10));
                    dl();
                }

                pci_device(session, bus, slot, data & 0xFFFF, (data >> 16) & 0xFFFF);

                dl();
            }
        }
    }
}
