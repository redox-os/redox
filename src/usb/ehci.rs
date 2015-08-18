use core::mem::size_of;

use common::memory::*;
use common::pci::*;

use programs::common::*;

struct STE {
    pub ptr: u64,
    pub length: u64
}

struct TRB {
    pub data: u64,
    pub status: u32,
    pub control: u32
}

impl TRB {
    pub fn new() -> TRB {
        TRB {
           data: 0,
           status: 0,
           control: 0
        }
    }

    pub fn from_type(trb_type: u32) -> TRB {
        TRB {
            data: 0,
            status: 0,
            control: (trb_type & 0x3F) << 10
        }
    }
}

pub struct EHCI {
    pub bus: usize,
    pub slot: usize,
    pub func: usize,
    pub base: usize,
    pub memory_mapped: bool,
    pub irq: u8
}

impl SessionModule for EHCI {
    fn on_irq(&mut self, irq: u8){
        if irq == self.irq {
            d("EHCI handle\n");
        }
    }
}

impl EHCI {
    pub unsafe fn init(&self){
        d("EHCI on: ");
        dh(self.base);
        if self.memory_mapped {
            d(" memory mapped");
        }else{
            d(" port mapped");
        }
        d(" IRQ: ");
        dbh(self.irq);

        pci_write(self.bus, self.slot, self.func, 0x04, pci_read(self.bus, self.slot, self.func, 0x04) | 4); // Bus master

        let USBCMD = self.base as *mut u32;
        let USBSTS = (self.base + 4) as *mut u32;
        let USBINTR = (self.base + 8) as *mut u32;
        let FRINDEX = (self.base + 0xC) as *mut u32;
        let CTRLDSSEGMENT = (self.base + 0x10) as *mut u32;
        let PERIODICLISTBASE = (self.base + 0x14) as *mut u32;
        let ASYNCLISTADDR = (self.base + 0x18) as *mut u32;
        let CONFIGFLAG = (self.base + 0x40) as *mut u32;
        let PORTSC = (self.base + 0x44) as *mut u32;

        //*CTRLDSSEGMENT = 0;

        //*USBINTR = 0b111111;

        //*PERIODICLISTBASE = alloc(4096) as u32;

        d(" CMD ");
        dh(*USBCMD as usize);

        d(" STS ");
        dh(*USBSTS as usize);

        dl();
    }
}
