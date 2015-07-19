use common::pio::*;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

pub unsafe fn pci_read(bus: usize, slot: usize, function: usize, offset: usize) -> usize{
    outd(CONFIG_ADDRESS, ((1 << 31) | (bus << 16) | (slot << 11) | (function << 8) | (offset & 0xfc)) as u32);
    return ind(CONFIG_DATA) as usize;
}

pub unsafe fn pci_write(bus: usize, slot: usize, function: usize, offset: usize, data: usize){
    outd(CONFIG_ADDRESS, ((1 << 31) | (bus << 16) | (slot << 11) | (function << 8) | (offset & 0xfc)) as u32);
    outd(CONFIG_DATA, data as u32);
}
