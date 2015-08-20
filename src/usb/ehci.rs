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
            //d("EHCI handle");

            unsafe{
                let CAPLENGTH = self.base as *mut u8;

                let opbase = self.base + *CAPLENGTH as usize;

                let USBSTS = (opbase + 4) as *mut u32;
                //d(" USBSTS ");
                //dh(*USBSTS as usize);

                *USBSTS = 0b111111;

                //d(" USBSTS ");
                //dh(*USBSTS as usize);

                //let FRINDEX = (opbase + 0xC) as *mut u32;
                //d(" FRINDEX ");
                //dh(*FRINDEX as usize);
            }

            //dl();
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

        let CAPLENGTH = self.base as *mut u8;

        d(" CAPLENGTH ");
        dd(*CAPLENGTH as usize);

        let opbase = self.base + *CAPLENGTH as usize;

        let USBCMD = opbase as *mut u32;
        let USBSTS = (opbase + 4) as *mut u32;
        let USBINTR = (opbase + 8) as *mut u32;
        let FRINDEX = (opbase + 0xC) as *mut u32;
        let CTRLDSSEGMENT = (opbase + 0x10) as *mut u32;
        let PERIODICLISTBASE = (opbase + 0x14) as *mut u32;
        let ASYNCLISTADDR = (opbase + 0x18) as *mut u32;
        let CONFIGFLAG = (opbase + 0x40) as *mut u32;
        let PORTSC = (opbase + 0x44) as *mut u32;

        *USBCMD &= 0xFFFFFFFE;
        d(" CMD ");
        dh(*USBCMD as usize);

        d(" STS ");
        dh(*USBSTS as usize);

        //*CTRLDSSEGMENT = 0;

        *USBINTR = 0b111111;

        let periodiclist = alloc(4096) as *mut u32;

        for i in 0..1024 {
            *periodiclist.offset(i) = periodiclist as u32 | 1;
        }
        *PERIODICLISTBASE = periodiclist as u32;

        *USBCMD |= 1;
        *CONFIGFLAG = 1;

        d(" CMD ");
        dh(*USBCMD as usize);

        d(" STS ");
        dh(*USBSTS as usize);

        dl();

        for i in 0..16 {
            if *PORTSC.offset(i) & 1 == 1 {
                d("Device on port ");
                dd(i as usize);
                dl();
            }
        }
    }
}
