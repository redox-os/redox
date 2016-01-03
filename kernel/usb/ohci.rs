use alloc::boxed::Box;

use collections::vec::Vec;

use core::mem;

use drivers::pci::config::PciConfig;

use schemes::KScheme;

use super::hci::{UsbHci, UsbMsg};
use super::setup::Setup;

#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
struct Gtd {
    flags: u32,
    buffer: u32,
    next: u32,
    end: u32,
}

pub struct Ohci {
    pub base: usize,
    pub irq: u8,
}

impl KScheme for Ohci {
    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            // d("OHCI IRQ\n");
        }
    }

    fn on_poll(&mut self) {
    }
}

impl Ohci {
    pub unsafe fn new(mut pci: PciConfig) -> Box<Self> {
        pci.flag(4, 4, true); // Bus mastering

        let mut module = box Ohci {
            base: pci.read(0x10) as usize & 0xFFFFFFF0,
            irq: pci.read(0x3C) as u8 & 0xF,
        };

        module.init();

        return module;
    }

    pub unsafe fn init(&mut self) {
        debugln!("OHCI on: {:X}, IRQ: {:X}", self.base, self.irq);
    }
}


impl UsbHci for Ohci {
    fn msg(&mut self, address: u8, endpoint: u8, msgs: &[UsbMsg]) -> usize {
        let mut tds = Vec::new();
        for msg in msgs.iter().rev() {
            let link_ptr = match tds.last() {
                Some(td) => (td as *const Gtd) as u32,
                None => 0
            };

            match *msg {
                UsbMsg::Setup(setup) => tds.push(Gtd {
                    flags: 0b00 << 19,
                    buffer: (setup as *const Setup) as u32,
                    next: link_ptr,
                    end: (setup as *const Setup) as u32 + mem::size_of::<Setup>() as u32
                }),
                UsbMsg::In(ref data) => tds.push(Gtd {
                    flags: 0b10 << 19,
                    buffer: data.as_ptr() as u32,
                    next: link_ptr,
                    end: data.as_ptr() as u32 + data.len() as u32
                }),
                UsbMsg::InIso(ref data) => tds.push(Gtd {
                    flags: 0b10 << 19,
                    buffer: data.as_ptr() as u32,
                    next: link_ptr,
                    end: data.as_ptr() as u32 + data.len() as u32
                }),
                UsbMsg::Out(ref data) => tds.push(Gtd {
                    flags: 0b01 << 19,
                    buffer: data.as_ptr() as u32,
                    next: link_ptr,
                    end: data.as_ptr() as u32 + data.len() as u32
                }),
                UsbMsg::OutIso(ref data) => tds.push(Gtd {
                    flags: 0b01 << 19,
                    buffer: data.as_ptr() as u32,
                    next: link_ptr,
                    end: data.as_ptr() as u32 + data.len() as u32
                })
            }
        }

        0
    }
}
