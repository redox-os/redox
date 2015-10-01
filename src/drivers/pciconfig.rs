use common::pio::*;

pub unsafe fn pci_read(bus: usize, slot: usize, function: usize, offset: usize) -> usize{
    let address = PIO32 { port: 0xCF8 };
    let data = PIO32 { port: 0xCFC };

    outd(CONFIG_ADDRESS, ((1 << 31) | (bus << 16) | (slot << 11) | (function << 8) | (offset & 0xfc)) as u32);
    return ind(CONFIG_DATA) as usize;
}

pub unsafe fn pci_write(bus: usize, slot: usize, function: usize, offset: usize, data: usize){
    let address = PIO32 { port: 0xCF8 };
    let data = PIO32 { port: 0xCFC };

    outd(CONFIG_ADDRESS, ((1 << 31) | (bus << 16) | (slot << 11) | (function << 8) | (offset & 0xfc)) as u32);
    outd(CONFIG_DATA, data as u32);
}
