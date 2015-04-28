use common::debug::*;
use common::pio::*;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

unsafe fn pci_read(bus: usize, slot: usize, function: usize, offset: usize) -> usize{
    outl(CONFIG_ADDRESS, ((1 << 31) | (bus << 16) | (slot << 11) | (function << 8) | (offset & 0xfc)) as u32);
    
    return inl(CONFIG_DATA) as usize;
}


pub unsafe fn net_test(dma: usize, port: usize){
    d("Network test\n");
    /*
    CTRL.LRST = 0;
    CTRL.PHY_RST = 0;
    CTRL.ILOS = 0;
    FCAH = 0;
    FCAL = 0;
    FCT = 0;
    FCTTV = 0;
    CTRL.VME = 0;
    
    RAL/RAH => Ethernet address in eeprom;
    
    MTA => 0;
    
    IMS => Enable interrupts;
    
    RDBAL/RDBAH => 16-byte aligned recieve buffer;
    RDLEN => size of buffer, 128-byte aligned ???;
    RDH => head address ???;
    RDT => tail address ???;
    */
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
                net_test(pci_read(0, device, 0, 0x10), pci_read(0, device, 0, 0x14));
            }
            
            dl();
        }
    }
}