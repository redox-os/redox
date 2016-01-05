use alloc::boxed::Box;

use collections::vec::Vec;

use core::intrinsics::volatile_load;
use core::mem;

use scheduler::context::context_switch;
use common::debug;
use common::memory::Memory;

use drivers::pci::config::PciConfig;
use drivers::pio::*;


use schemes::KScheme;

use super::{Hci, Packet, Pipe, Setup};

pub struct Uhci {
    pub base: usize,
    pub irq: u8,
    pub frame_list: Memory<u32>,
}

impl KScheme for Uhci {
    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            // d("UHCI IRQ\n");
        }
    }

    fn on_poll(&mut self) {
    }
}

#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
struct Td {
    link_ptr: u32,
    ctrl_sts: u32,
    token: u32,
    buffer: u32,
}

#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
struct Qh {
    head_ptr: u32,
    element_ptr: u32,
}

impl Uhci {
    pub unsafe fn new(mut pci: PciConfig) -> Box<Self> {
        pci.flag(4, 4, true); // Bus mastering

        let mut module = box Uhci {
            base: pci.read(0x20) as usize & 0xFFFFFFF0,
            irq: pci.read(0x3C) as u8 & 0xF,
            frame_list: Memory::new(1024).unwrap(),
        };

        module.init();

        return module;
    }

    pub unsafe fn init(&mut self) {
        debugln!("UHCI on: {:X}, IRQ: {:X}", self.base, self.irq);

        let base = self.base as u16;
        let usbcmd = base;
        let usbsts = base + 02;
        let usbintr = base + 0x4;
        let frnum = base + 0x6;
        let flbaseadd = base + 0x8;
        let portsc1 = base + 0x10;
        let portsc2 = base + 0x12;

        debug::d(" CMD ");
        debug::dh(inw(usbcmd) as usize);
        outw(usbcmd, 1 << 2 | 1 << 1);
        debug::d(" to ");
        debug::dh(inw(usbcmd) as usize);

        outw(usbcmd, 0);
        debug::d(" to ");
        debug::dh(inw(usbcmd) as usize);

        debug::d(" STS ");
        debug::dh(inw(usbsts) as usize);

        debug::d(" INTR ");
        debug::dh(inw(usbintr) as usize);

        debug::d(" FRNUM ");
        debug::dh(inw(frnum) as usize);
        outw(frnum, 0);
        debug::d(" to ");
        debug::dh(inw(frnum) as usize);

        debug::d(" FLBASEADD ");
        debug::dh(ind(flbaseadd) as usize);
        for i in 0..1024 {
            self.frame_list.write(i, 1);
        }
        outd(flbaseadd, self.frame_list.address() as u32);
        debug::d(" to ");
        debug::dh(ind(flbaseadd) as usize);

        debug::d(" CMD ");
        debug::dh(inw(usbcmd) as usize);
        outw(usbcmd, 1);
        debug::d(" to ");
        debug::dh(inw(usbcmd) as usize);

        debug::dl();

        {
            debug::d(" PORTSC1 ");
            debug::dh(inw(portsc1) as usize);

            outw(portsc1, 1 << 9);
            debug::d(" to ");
            debug::dh(inw(portsc1) as usize);

            outw(portsc1, 0);
            debug::d(" to ");
            debug::dh(inw(portsc1) as usize);

            debug::dl();

            if inw(portsc1) & 1 == 1 {
                debug::d(" Device Found ");
                debug::dh(inw(portsc1) as usize);

                outw(portsc1, 4);
                debug::d(" to ");
                debug::dh(inw(portsc1) as usize);
                debug::dl();

                self.device(1);
            }
        }

        {
            debug::d(" PORTSC2 ");
            debug::dh(inw(portsc2) as usize);

            outw(portsc2, 1 << 9);
            debug::d(" to ");
            debug::dh(inw(portsc2) as usize);

            outw(portsc2, 0);
            debug::d(" to ");
            debug::dh(inw(portsc2) as usize);

            debug::dl();

            if inw(portsc2) & 1 == 1 {
                debug::d(" Device Found ");
                debug::dh(inw(portsc2) as usize);

                outw(portsc2, 4);
                debug::d(" to ");
                debug::dh(inw(portsc2) as usize);
                debug::dl();

                self.device(2);
            }
        }
    }
}

impl Hci for Uhci {
    fn msg(&mut self, address: u8, endpoint: u8, pipe: Pipe, msgs: &[Packet]) -> usize {
        let ctrl_sts = match pipe {
            Pipe::Isochronous => 1 << 25 | 1 << 23,
            _ => 1 << 23
        };

        let mut tds = Vec::new();
        for msg in msgs.iter().rev() {
            let link_ptr = match tds.last() {
                Some(td) => (td as *const Td) as u32 | 4,
                None => 1
            };

            match *msg {
                Packet::Setup(setup) => tds.push(Td {
                    link_ptr: link_ptr,
                    ctrl_sts: ctrl_sts,
                    token: (mem::size_of::<Setup>() as u32 - 1) << 21 | (endpoint as u32) << 15 | (address as u32) << 8 | 0x2D,
                    buffer: (&*setup as *const Setup) as u32,
                }),
                Packet::In(ref data) => tds.push(Td {
                    link_ptr: link_ptr,
                    ctrl_sts: ctrl_sts,
                    token: ((data.len() as u32 - 1) & 0x7FF) << 21 | (endpoint as u32) << 15 | (address as u32) << 8 | 0x69,
                    buffer: data.as_ptr() as u32,
                }),
                Packet::Out(ref data) => tds.push(Td {
                    link_ptr: link_ptr,
                    ctrl_sts: ctrl_sts,
                    token: ((data.len() as u32 - 1) & 0x7FF) << 21 | (endpoint as u32) << 15 | (address as u32) << 8 | 0xE1,
                    buffer: data.as_ptr() as u32,
                })
            }
        }

        let mut count = 0;

        if ! tds.is_empty() {
            let queue_head = box Qh {
                 head_ptr: 1,
                 element_ptr: (tds.last().unwrap() as *const Td) as u32,
            };

            let frame_ptr = if tds.len() == 1 {
                (&tds[0] as *const Td) as u32
            } else {
                (&*queue_head as *const Qh) as u32 | 2
            };

            let frnum = Pio16::new(self.base as u16 + 6);
            let frame = (unsafe { frnum.read() } + 1) & 0x3FF;
            unsafe { self.frame_list.write(frame as usize, frame_ptr) };

            for td in tds.iter().rev() {
                while unsafe { volatile_load(td as *const Td).ctrl_sts } & 1 << 23 == 1 << 23 {
                    unsafe { context_switch(false) };
                }
                count += (unsafe { volatile_load(td as *const Td).ctrl_sts } & 0x7FF) as usize;
            }

            unsafe { self.frame_list.write(frame as usize, 1) };
        }

        count
    }
}
