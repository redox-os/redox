use alloc::boxed::Box;

use collections::vec::Vec;

use core::mem;

use arch::context::context_switch;
// use common::debug;
use arch::memory::Memory;

use drivers::pci::config::PciConfig;
use drivers::io::{Io, Mmio, PhysAddr, Pio};

use fs::KScheme;

use super::{Hci, Packet, Pipe, Setup};

pub struct Uhci {
    pub base: usize,
    pub irq: u8,
    pub frame_list: Memory<PhysAddr<Mmio<u32>>>,
}

impl KScheme for Uhci {
    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            // d("UHCI IRQ\n");
        }
    }
}

bitflags! {
    flags LinkFlags: u32 {
        /// The link is not valid, if set
        const LINK_TERMINATE = 1,
        /// The link references a queue head, if set.
        /// Otherwise, it references a transfer descriptor
        const LINK_QH_SELECT = 2,
        /// The link should be processed depth first, if set.
        /// Otherwise, it should be processed breadth first
        const LINK_DEPTH_SELECT = 4
    }
}

bitflags! {
    flags CtrlStsFlags: u32 {
        const CTRL_SHORT_PACKET_DETECT = 1 << 29,
        const CTRL_LOW_SPEED = 1 << 26,
        const CTRL_ISOCHRONOUS = 1 << 25,
        const CTRL_INTERRUPT = 1 << 24,
        const STS_ACTIVE = 1 << 23,
        const STS_STALLED = 1 << 22,
        const STS_BUFFER_ERROR = 1 << 21,
        const STS_BABBLE = 1 << 20,
        const STS_NAK = 1 << 19,
        const STS_TIMEOUT = 1 << 18,
        const STS_BITSTUFF = 1 << 17
    }
}

#[repr(packed)]
struct Td {
    link_ptr: PhysAddr<Mmio<u32>>,
    ctrl_sts: Mmio<u32>,
    token: Mmio<u32>,
    buffer: PhysAddr<Mmio<u32>>,
}

#[repr(packed)]
struct Qh {
    head_ptr: PhysAddr<Mmio<u32>>,
    element_ptr: PhysAddr<Mmio<u32>>,
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
            self.frame_list[i].write(LINK_TERMINATE.bits);
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

impl Hci for Uhci {
    fn msg(&mut self, address: u8, endpoint: u8, pipe: Pipe, msgs: &[Packet]) -> usize {
        let mut tds = Vec::new();
        for msg in msgs.iter().rev() {
            let mut td = Td {
                link_ptr: PhysAddr::new(Mmio::new()),
                ctrl_sts: Mmio::new(),
                token: Mmio::new(),
                buffer: PhysAddr::new(Mmio::new()),
            };

            td.link_ptr.write(match tds.last() {
                Some(last_td) => (last_td as *const Td) as u32 | LINK_DEPTH_SELECT.bits,
                None => LINK_TERMINATE.bits,
            });

            td.ctrl_sts.write(match pipe {
                Pipe::Isochronous => (CTRL_ISOCHRONOUS | STS_ACTIVE).bits,
                _ => STS_ACTIVE.bits,
            });

            match *msg {
                Packet::Setup(setup) => {
                    td.token.write((mem::size_of::<Setup>() as u32 - 1) << 21 |
                                   (endpoint as u32) << 15 |
                                   (address as u32) << 8 | 0x2D);
                    td.buffer.write((&*setup as *const Setup) as u32);
                },
                Packet::In(ref data) => {
                    td.token
                        .write(((data.len() as u32 - 1) & 0x7FF) << 21 | (endpoint as u32) << 15 |
                               (address as u32) << 8 | 0x69);
                    td.buffer.write(data.as_ptr() as u32);
                },
                Packet::Out(ref data) => {
                    td.token
                        .write(((data.len() as u32 - 1) & 0x7FF) << 21 | (endpoint as u32) << 15 |
                               (address as u32) << 8 | 0xE1);
                    td.buffer.write(data.as_ptr() as u32);
                },
            }

            tds.push(td);
        }

        let mut count = 0;

        if !tds.is_empty() {
            let mut queue_head = box Qh {
                head_ptr: PhysAddr::new(Mmio::new()),
                element_ptr: PhysAddr::new(Mmio::new()),
            };

            queue_head.head_ptr.write(LINK_TERMINATE.bits);
            queue_head.element_ptr.write((tds.last().unwrap() as *const Td) as u32);

            let frame_ptr = if tds.len() == 1 {
                (&tds[0] as *const Td) as u32
            } else {
                (&*queue_head as *const Qh) as u32 | LINK_QH_SELECT.bits
            };

            let frnum = Pio::<u16>::new(self.base as u16 + 6);
            let frame = (frnum.read() + 1) & 0x3FF;
            self.frame_list[frame as usize].write(frame_ptr);

            for td in tds.iter().rev() {
                while td.ctrl_sts.read() & STS_ACTIVE.bits == STS_ACTIVE.bits {
                    unsafe { context_switch() };
                }
                count += td.ctrl_sts.read() & 0x7FF;
            }

            self.frame_list[frame as usize].write(LINK_TERMINATE.bits);
        }

        count as usize
    }
}
