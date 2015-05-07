use common::debug::*;
use common::pio::*;

use network::intel8254x::*;
use network::rtl8139::*;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

unsafe fn pci_read(bus: usize, slot: usize, function: usize, offset: usize) -> usize{
    outd(CONFIG_ADDRESS, ((1 << 31) | (bus << 16) | (slot << 11) | (function << 8) | (offset & 0xfc)) as u32);
    return ind(CONFIG_DATA) as usize;
}

unsafe fn pci_write(bus: usize, slot: usize, function: usize, offset: usize, data: usize){
    outd(CONFIG_ADDRESS, ((1 << 31) | (bus << 16) | (slot << 11) | (function << 8) | (offset & 0xfc)) as u32);
    outd(CONFIG_DATA, data as u32);
}

pub unsafe fn pci_handle(irq: u8){
    d("PCI Handle ");
    dh(irq as usize);
    dl();

    for device in 0..32 {
        let data = pci_read(0, device, 0, 0);

        if (data & 0xFFFF) != 0xFFFF {
            if irq == pci_read(0, device, 0, 0x3C) as u8 & 0xF {
                if data == 0x100E8086 {
                    let base = pci_read(0, device, 0, 0x10);
                    let device = Intel8254x {
                        base: base & (0xFFFFFFFF - 1),
                        memory_mapped: base & 1 == 0
                    };
                    device.handle();
                } else if data == 0x813910EC {
                    let base = pci_read(0, device, 0, 0x10);
                    let device = RTL8139 {
                        base: base & (0xFFFFFFFF - 1),
                        memory_mapped: base & 1 == 0
                    };
                    device.handle();
                }
            }
        }
    }
}

pub unsafe fn pci_test(){
    d("PCI\n");

    for device in 0..32 {
        let data = pci_read(0, device, 0, 0);

        if (data & 0xFFFF) != 0xFFFF {
            d("Device ");
            dd(device);
            d(": ");
            dh(data);
            d(", ");
            dh(pci_read(0, device, 0, 8));
            dl();

            for i in 0..6 {
                d("    ");
                dd(i);
                d(": ");
                dh(pci_read(0, device, 0, i*4 + 0x10));
                dl();
            }

            if data == 0x100E8086 {
                let base = pci_read(0, device, 0, 0x10);
                let device = Intel8254x {
                    base: base & (0xFFFFFFFF - 1),
                    memory_mapped: base & 1 == 0
                };
                device.init();
            } else if data == 0x813910EC {
                pci_write(0, device, 0, 0x04, pci_read(0, device, 0, 0x04) | (1 << 2));

                d("IRQ ");
                dh(pci_read(0, device, 0, 0x3C) & 0xF + 0x20);
                dl();

                let base = pci_read(0, device, 0, 0x10);
                let device = RTL8139 {
                    base: base & (0xFFFFFFFF - 1),
                    memory_mapped: base & 1 == 0
                };
                device.init();
            }

            dl();
        }
    }
}