use alloc::boxed::Box;

use collections::vec::Vec;

use core::intrinsics::volatile_load;
use core::mem;

use arch::context::context_switch;
//use common::debug;
use arch::memory::Memory;

use drivers::pci::config::PciConfig;
use drivers::io::{Io, Pio};

use fs::KScheme;

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
            frame_list: Memory::new_aligned(1024, 4096).unwrap(),
        };

        module.init();

        return module;
    }

    pub unsafe fn init(&mut self) {
        debugln!(" + UHCI on: {:X}, IRQ: {:X}", self.base, self.irq);

        let base = self.base as u16;
        let mut usbcmd = Pio::<u16>::new(base);
        let usbsts = Pio::<u16>::new(base + 0x2);
        let usbintr = Pio::<u16>::new(base + 0x4);
        let mut frnum = Pio::<u16>::new(base + 0x6);
        let mut flbaseadd = Pio::<u32>::new(base + 0x8);
        let mut portscs = [Pio::<u16>::new(base + 0x10), Pio::<u16>::new(base + 0x12)];

        debug!(" CMD {:X}", usbcmd.read());
        usbcmd.write(1 << 2 | 1 << 1);
        debug!(" to {:X}", usbcmd.read());
        usbcmd.write(0);
        debug!(" to {:X}", usbcmd.read());

        debug!(" STS {:X}", usbsts.read());

        debug!(" INTR {:X}", usbintr.read());

        debug!(" FRNUM {:X}", frnum.read());
        frnum.write(0);
        debug!(" to {:X}", frnum.read());

        debug!(" FLBASEADD {:X}", flbaseadd.read());
        for i in 0..1024 {
            self.frame_list.store(i, 1);
        }
        flbaseadd.write(self.frame_list.address() as u32);
        debug!(" to {:X}", flbaseadd.read());

        debug!(" CMD {:X}", usbcmd.read());
        usbcmd.write(1);
        debug!(" to {:X}", usbcmd.read());

        debugln!("");

        for i in 0..portscs.len() {
            let portsc = &mut portscs[i];

            debug!(" PORTSC{} {:X}", i + 1, portsc.read());

            portsc.write(1 << 9);
            debug!(" to {:X}", portsc.read());

            portsc.write(0);
            debugln!(" to {:X}", portsc.read());

            if portsc.read() & 1 == 1 {
                debug!(" Device Found {:X}", portsc.read());

                portsc.write(4);
                debugln!(" to {:X}", portsc.read());

                self.device((i + 1) as u8);
            }
        }
    }
}

fn convert_phys(ptr: u32) -> u32 {
    if ptr >= 0x80000000 {
        ptr - 0x80000000
    } else {
        ptr
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
                Some(td) => convert_phys((td as *const Td) as u32) | 4,
                None => 1
            };

            match *msg {
                Packet::Setup(setup) => tds.push(Td {
                    link_ptr: link_ptr,
                    ctrl_sts: ctrl_sts,
                    token: (mem::size_of::<Setup>() as u32 - 1) << 21 | (endpoint as u32) << 15 | (address as u32) << 8 | 0x2D,
                    buffer: convert_phys((&*setup as *const Setup) as u32),
                }),
                Packet::In(ref data) => tds.push(Td {
                    link_ptr: link_ptr,
                    ctrl_sts: ctrl_sts,
                    token: ((data.len() as u32 - 1) & 0x7FF) << 21 | (endpoint as u32) << 15 | (address as u32) << 8 | 0x69,
                    buffer: convert_phys(data.as_ptr() as u32),
                }),
                Packet::Out(ref data) => tds.push(Td {
                    link_ptr: link_ptr,
                    ctrl_sts: ctrl_sts,
                    token: ((data.len() as u32 - 1) & 0x7FF) << 21 | (endpoint as u32) << 15 | (address as u32) << 8 | 0xE1,
                    buffer: convert_phys(data.as_ptr() as u32),
                })
            }
        }

        let mut count = 0;

        if ! tds.is_empty() {
            let queue_head = box Qh {
                 head_ptr: 1,
                 element_ptr: convert_phys((tds.last().unwrap() as *const Td) as u32),
            };

            let frame_ptr = if tds.len() == 1 {
                (&tds[0] as *const Td) as u32
            } else {
                (&*queue_head as *const Qh) as u32 | 2
            };

            let frnum = Pio::<u16>::new(self.base as u16 + 6);
            let frame = (frnum.read() + 1) & 0x3FF;
            self.frame_list.store(frame as usize, convert_phys(frame_ptr));

            for td in tds.iter().rev() {
                while unsafe { volatile_load(td as *const Td).ctrl_sts } & 1 << 23 == 1 << 23 {
                    unsafe { context_switch() };
                }
                count += unsafe { volatile_load(td as *const Td).ctrl_sts } & 0x7FF;
            }

            self.frame_list.store(frame as usize, 1);
        }

        count as usize
    }
}
