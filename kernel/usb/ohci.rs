use alloc::boxed::Box;

use collections::vec::Vec;

use core::intrinsics::volatile_load;
use core::mem;

use drivers::io::{Io, Mmio};
use drivers::pci::config::PciConfig;

use arch::context::context_switch;

use fs::KScheme;

use super::{Hci, Packet, Pipe, Setup};

#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
struct Gtd {
    flags: u32,
    buffer: u32,
    next: u32,
    end: u32,
}

#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
struct Ed {
    flags: u32,
    tail: u32,
    head: u32,
    next: u32,
}

const CTRL_CBSR: u32 = 0b11;
const CTRL_PLE: u32 = 1 << 2;
const CTRL_IE: u32 = 1 << 3;
const CTRL_CLE: u32 = 1 << 4;
const CTRL_BLE: u32 = 1 << 5;
const CTRL_HCFS: u32 = 0b11 << 6;
const CTRL_IR: u32 = 1 << 8;
const CTRL_RWC: u32 = 1 << 9;
const CTRL_RWE: u32 = 1 << 10;

const CMD_STS_HCR: u32 = 1;
const CMD_STS_CLF: u32 = 1 << 1;
const CMD_STS_BLF: u32 = 1 << 2;
const CMD_STS_OCR: u32 = 1 << 3;

const PORT_STS_CCS: u32 = 1;
const PORT_STS_PES: u32 = 1 << 1;
const PORT_STS_PSS: u32 = 1 << 2;
const PORT_STS_POCI: u32 = 1 << 3;
const PORT_STS_PPS: u32 = 1 << 8;
const PORT_STS_LSDA: u32 = 1 << 9;
const PORT_STS_CSC: u32 = 1 << 16;
const PORT_STS_PESC: u32 = 1 << 17;
const PORT_STS_PSSC: u32 = 1 << 18;
const PORT_STS_OCIC: u32 = 1 << 19;
const PORT_STS_PRSC: u32 = 1 << 20;

#[repr(packed)]
pub struct OhciRegs {
    pub revision: Mmio<u32>,
    pub control: Mmio<u32>,
    pub cmd_sts: Mmio<u32>,
    pub int_sts: Mmio<u32>,
    pub int_en: Mmio<u32>,
    pub int_dis: Mmio<u32>,
    pub hcca: Mmio<u32>,
    pub period_current: Mmio<u32>,
    pub control_head: Mmio<u32>,
    pub control_current: Mmio<u32>,
    pub bulk_head: Mmio<u32>,
    pub bulk_current: Mmio<u32>,
    pub done_head: Mmio<u32>,
    pub fm_interval: Mmio<u32>,
    pub fm_remain: Mmio<u32>,
    pub fm_num: Mmio<u32>,
    pub periodic_start: Mmio<u32>,
    pub ls_thresh: Mmio<u32>,
    pub rh_desc_a: Mmio<u32>,
    pub rh_desc_b: Mmio<u32>,
    pub rh_sts: Mmio<u32>,
    pub port_sts: [Mmio<u32>; 15],
}

#[repr(packed)]
pub struct OhciHcca {
    pub interrupt_table: [u32; 32],
    pub frame_number: u16,
    pub padding: u16,
    pub done_head: u32,
    pub reserved: [u8; 116],
}

pub struct Ohci {
    pub regs: &'static mut OhciRegs,
    pub hcca: Box<OhciHcca>,
    pub irq: u8,
}

impl KScheme for Ohci {
    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            // d("OHCI IRQ\n");
        }
    }
}

impl Ohci {
    pub unsafe fn new(mut pci: PciConfig) -> Box<Self> {
        pci.flag(4, 4, true); // Bus mastering

        let base = pci.read(0x10) as usize & 0xFFFFFFF0;
        let regs = &mut *(base as *mut OhciRegs);

        let mut module = box Ohci {
            regs: regs,
            hcca: box OhciHcca {
                interrupt_table: [0; 32],
                frame_number: 0,
                padding: 0,
                done_head: 0,
                reserved: [0; 116],
            },
            irq: pci.read(0x3C) as u8 & 0xF,
        };

        module.init();

        return module;
    }

    pub unsafe fn init(&mut self) {
        debugln!(" + OHCI on: {:X}, IRQ: {:X}",
                 (self.regs as *mut OhciRegs) as usize,
                 self.irq);

        // self.regs.hcca.write((&*self.hcca as *const OhciHcca) as u32);
        //
        // debugln!("Enable: {:X}", self.regs.control.read());
        // loop {
        // let ctrl = self.regs.control.read();
        // let desired_ctrl = (ctrl & (0xFFFFFFFF - CTRL_HCFS)) | 0b10 << 6;
        // if ctrl != desired_ctrl {
        // self.regs.control.write(desired_ctrl);
        // } else {
        // break;
        // }
        // }
        //
        // debugln!("CTRL: {:X} CMDSTS: {:X} HCCA: {:X}", self.regs.control.read(),
        // self.regs.cmd_sts.read(), self.regs.hcca.read());
        //
        // let ndp = self.regs.rh_desc_a.read() & 0xF;
        // for i in 0..ndp as usize {
        // debugln!("Port {}: {:X}", i, self.regs.port_sts[i].read());
        //
        // if self.regs.port_sts[i].readf(PORT_STS_CCS) {
        // debugln!("  Device Found");
        //
        // while ! self.regs.port_sts[i].readf(PORT_STS_PES) {
        // self.regs.port_sts[i].writef(PORT_STS_PES, true);
        // }
        //
        // self.device(i as u8 + 1);
        // }
        // }
        //
    }
}


impl Hci for Ohci {
    fn msg(&mut self, address: u8, endpoint: u8, _pipe: Pipe, msgs: &[Packet]) -> usize {
        let mut tds = Vec::new();
        for msg in msgs.iter() {
            match *msg {
                Packet::Setup(setup) => {
                    tds.push(Gtd {
                        flags: 0b1111 << 28 | 0b00 << 19 | 1 << 18,
                        buffer: (setup as *const Setup) as u32,
                        next: 0,
                        end: (setup as *const Setup) as u32 + mem::size_of::<Setup>() as u32 - 1,
                    })
                },
                Packet::In(ref data) => {
                    tds.push(Gtd {
                        flags: 0b1111 << 28 | 0b10 << 19 | 1 << 18,
                        buffer: if data.is_empty() {
                            0
                        } else {
                            data.as_ptr() as u32
                        },
                        next: 0,
                        end: if data.is_empty() {
                            0
                        } else {
                            data.as_ptr() as u32 + data.len() as u32 - 1
                        },
                    })
                },
                Packet::Out(ref data) => {
                    tds.push(Gtd {
                        flags: 0b1111 << 28 | 0b01 << 19 | 1 << 18,
                        buffer: if data.is_empty() {
                            0
                        } else {
                            data.as_ptr() as u32
                        },
                        next: 0,
                        end: if data.is_empty() {
                            0
                        } else {
                            data.as_ptr() as u32 + data.len() as u32 - 1
                        },
                    })
                },
            }
        }

        let mut count = 0;

        if !tds.is_empty() {
            for i in 0..tds.len() - 1 {
                tds[i].next = (&tds[i + 1] as *const Gtd) as u32;
            }

            let ed = box Ed {
                flags: 8 << 16 | (endpoint as u32) << 7 | address as u32,
                tail: (tds.last().unwrap() as *const Gtd) as u32 + mem::size_of::<Gtd>() as u32,
                head: (tds.first().unwrap() as *const Gtd) as u32,
                next: 0,
            };

            // debugln!("ED: {:X}, FLG: {:X}, TAIL: {:X}, HEAD: {:X}, NEXT: {:X}", (&*ed as
            // *const Ed) as usize, ed.flags, ed.tail, ed.head, ed.next);

            while !self.regs.control.readf(CTRL_CLE) {
                self.regs.control.writef(CTRL_CLE, true);
            }
            self.regs.control_head.write((&*ed as *const Ed) as u32);
            while !self.regs.cmd_sts.readf(CMD_STS_CLF) {
                self.regs.cmd_sts.writef(CMD_STS_CLF, true);
            }

            for td in tds.iter() {
                // debugln!("  TD: {:X}, FLG: {:X}, BUF: {:X}, NEXT: {:X}, END: {:X}", (td as
                // *const Gtd) as usize, td.flags, td.buffer, td.next, td.end);

                while unsafe { volatile_load(td as *const Gtd).flags } & 0b1111 << 28 ==
                      0b1111 << 28 {
                    unsafe { context_switch() };
                }

                let condition = (unsafe { volatile_load(td as *const Gtd).flags } &
                                 0b1111 << 28) >> 28;
                if condition != 0 {
                    // debugln!("  /TD: {:X}, FLG: {:X}, BUF: {:X}, NEXT: {:X}, END: {:X}", (td as
                    // *const Gtd) as usize, td.flags, td.buffer, td.next, td.end);
                    debugln!("Condition: {:X}", condition);
                    break;
                } else {
                    count += (td.end - td.buffer) as usize;
                }
            }

            // while self.regs.cmd_sts.readf(CMD_STS_CLF) {
            // self.regs.cmd_sts.writef(CMD_STS_CLF, false);
            // }
            //
            while self.regs.control.readf(CTRL_CLE) {
                self.regs.control.writef(CTRL_CLE, false);
            }
            self.regs.control_head.write(0);

            // debugln!("/ED: {:X}, FLG: {:X}, TAIL: {:X}, HEAD: {:X}, NEXT: {:X}", (&*ed
            // as *const Ed) as usize, ed.flags, ed.tail, ed.head, ed.next);
        }

        count
    }
}
