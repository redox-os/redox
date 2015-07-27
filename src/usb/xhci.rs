use common::debug::*;
use common::pci::*;

use programs::session::*;

pub struct XHCI {
    pub bus: usize,
    pub slot: usize,
    pub func: usize,
    pub base: usize,
    pub memory_mapped: bool,
    pub irq: u8
}

impl SessionDevice for XHCI {
    fn handle(&mut self, irq: u8){
        if irq == self.irq {
            d("XHCI handle");
        }
    }
}

impl XHCI {
    pub unsafe fn init(&self){
        d("XHCI on: ");
        dh(self.base);
        if self.memory_mapped {
            d(" memory mapped");
        }else{
            d(" port mapped");
        }
        d(" IRQ: ");
        dbh(self.irq);
        dl();

        pci_write(self.bus, self.slot, self.func, 0x04, pci_read(self.bus, self.slot, self.func, 0x04) | (1 << 2)); // Bus mastering

        //let base = self.base as u16;
    }
}
