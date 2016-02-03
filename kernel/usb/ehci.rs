use alloc::boxed::Box;

use collections::vec::Vec;

use core::intrinsics::volatile_load;
use core::mem::size_of;
use core::slice;

use common::debug;

use drivers::io::{Io, Mmio};
use drivers::pci::config::PciConfig;

use arch::context::context_switch;

use schemes::KScheme;

use super::{Hci, Packet, Pipe, Setup};

#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
struct Qtd {
    next: u32,
    next_alt: u32,
    token: u32,
    buffers: [u32; 5],
}

#[repr(packed)]
struct QueueHead {
    next: u32,
    characteristics: u32,
    capabilities: u32,
    qtd_ptr: u32,
    qtd: Qtd,
}

pub struct Ehci {
    pub pci: PciConfig,
    pub base: usize,
    pub irq: u8,
}

impl KScheme for Ehci {
    #[allow(non_snake_case)]
    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            // debug::d("EHCI handle");

            unsafe {
                let cap_length = &mut *(self.base as *mut Mmio<u8>);

                let op_base = self.base + cap_length.read() as usize;

                let usb_sts = &mut *((op_base + 4) as *mut Mmio<u32>);
                // debug::d(" usb_sts ");
                // debug::dh(*usb_sts as usize);

                usb_sts.writef(0b111111, true);

                // debug::d(" usb_sts ");
                // debug::dh(*usb_sts as usize);

                // let FRINDEX = (opbase + 0xC) as *mut Mmio<u32>;
                // debug::d(" FRINDEX ");
                // debug::dh(*FRINDEX as usize);
            }

            // debug::dl();
        }
    }
}

impl Ehci {
    pub unsafe fn new(mut pci: PciConfig) -> Box<Self> {
        let mut module = box Ehci {
            pci: pci,
            base: pci.read(0x10) as usize & 0xFFFFFFF0,
            irq: pci.read(0x3C) as u8 & 0xF,
        };

        module.init();

        module
    }

    #[allow(non_snake_case)]
    pub unsafe fn init(&mut self) {
        debug!("EHCI on: {:X}, IRQ {:X}", self.base, self.irq);

        self.pci.flag(4, 4, true); // Bus master

        let cap_length = &mut *(self.base as *mut Mmio<u8>);
        let hcs_params = &mut *((self.base + 4) as *mut Mmio<u32>);
        let hcc_params = &mut *((self.base + 8) as *mut Mmio<u32>);

        let ports = (hcs_params.read() & 0b1111) as usize;
        debug::d(" PORTS ");
        debug::dd(ports);

        let eecp = (hcc_params.read() >> 8) as u8;
        debug::d(" EECP ");
        debug::dh(eecp as usize);

        debug::dl();

        if eecp > 0 {
            if self.pci.read(eecp) & (1 << 24 | 1 << 16) == 1 << 16 {
                debug::d("Taking Ownership");
                debug::d(" ");
                debug::dh(self.pci.read(eecp) as usize);

                self.pci.flag(eecp, 1 << 24, true);

                debug::d(" ");
                debug::dh(self.pci.read(eecp) as usize);
                debug::dl();

                debug::d("Waiting");
                debug::d(" ");
                debug::dh(self.pci.read(eecp) as usize);

                while self.pci.read(eecp) & (1 << 24 | 1 << 16) != 1 << 24 {}

                debug::d(" ");
                debug::dh(self.pci.read(eecp) as usize);
                debug::dl();
            }
        }

        let op_base = self.base + cap_length.read() as usize;

        let usb_cmd = &mut *(op_base as *mut Mmio<u32>);
        let usb_sts = &mut *((op_base + 4) as *mut Mmio<u32>);
        let usb_intr = &mut *((op_base + 8) as *mut Mmio<u32>);
        let config_flag = &mut *((op_base + 0x40) as *mut Mmio<u32>);
        let port_scs = &mut slice::from_raw_parts_mut((op_base + 0x44) as *mut Mmio<u32>, ports);

        /*
        let FRINDEX = (opbase + 0xC) as *mut Mmio<u32>;
        let CTRLDSSEGMENT = (opbase + 0x10) as *mut Mmio<u32>;
        let PERIODICLISTBASE = (opbase + 0x14) as *mut Mmio<u32>;
        let ASYNCLISTADDR = (opbase + 0x18) as *mut Mmio<u32>;
        */

        //Halt
        if usb_sts.read() & 1 << 12 == 0 {
            usb_cmd.writef(0xF, false);
            while ! usb_sts.readf(1 << 12) {}
        }

        //Reset
        usb_cmd.writef(1 << 1, true);
        while usb_cmd.readf(1 << 1) {}

        //Enable
        usb_intr.write(0b111111);
        usb_cmd.writef(1, true);
        config_flag.write(1);
        while usb_sts.readf(1 << 12) {}

        for i in 0..port_scs.len() {
            let port_sc = &mut port_scs[i];
            debugln!("Port {}: {:X}", i, port_sc.read());

            if port_sc.readf(1) {
                debugln!("Device Found");

                if port_sc.readf(1 << 1) {
                    debugln!("Connection Change");

                    port_sc.writef(1 << 1, true);
                }

                if ! port_sc.readf(1 << 2) {
                    debugln!("Reset");

                    while ! port_sc.readf(1 << 8) {
                        port_sc.writef(1 << 8, true);
                    }

                    let mut spin = 1000000000;
                    while spin > 0 {
                        spin -= 1;
                    }

                    while port_sc.readf(1 << 8) {
                        port_sc.writef(1 << 8, false);
                    }
                }

                debugln!("Port Enabled {:X}", port_sc.read());

                self.device(i as u8 + 1);
            }
        }
    }
}

impl Hci for Ehci {
    fn msg(&mut self, address: u8, endpoint: u8, _pipe: Pipe, msgs: &[Packet]) -> usize {
        let mut tds = Vec::new();
        for msg in msgs.iter().rev() {
            let link_ptr = match tds.last() {
                Some(td) => (td as *const Qtd) as u32,
                None => 1
            };

            match *msg {
                Packet::Setup(setup) => tds.push(Qtd {
                    next: link_ptr,
                    next_alt: 1,
                    token: (size_of::<Setup>() as u32) << 16 | 0b10 << 8 | 1 << 7,
                    buffers: [(setup as *const Setup) as u32, 0, 0, 0, 0]
                }),
                Packet::In(ref data) => tds.push(Qtd {
                    next: link_ptr,
                    next_alt: 1,
                    token: ((data.len() as u32) & 0x7FFF) << 16 | 0b01 << 8 | 1 << 7,
                    buffers: [if data.is_empty() {
                        0
                    } else {
                        data.as_ptr() as u32
                    }, 0, 0, 0, 0]
                }),
                Packet::Out(ref data) => tds.push(Qtd {
                    next: link_ptr,
                    next_alt: 1,
                    token: ((data.len() as u32) & 0x7FFF) << 16 | 0b00 << 8 | 1 << 7,
                    buffers: [if data.is_empty() {
                        0
                    } else {
                        data.as_ptr() as u32
                    }, 0, 0, 0, 0]
                })
            }
        }

        let mut count = 0;

        if ! tds.is_empty() {
            unsafe {
                let cap_length = &mut *(self.base as *mut Mmio<u8>);

                let op_base = self.base + cap_length.read() as usize;

                let usb_cmd = &mut *(op_base as *mut Mmio<u32>);
                let async_list = &mut *((op_base + 0x18) as *mut Mmio<u32>);

                let queuehead = box QueueHead {
                    next: 1,
                    characteristics: 64 << 16 | 1 << 15 | 1 << 14 | 0b10 << 12 | (endpoint as u32) << 8 | address as u32,
                    capabilities: 0b01 << 30,
                    qtd_ptr: (tds.last().unwrap() as *const Qtd) as u32,
                    qtd: *tds.last().unwrap()
                };

                //TODO: Calculate actual bytes
                for td in tds.iter().rev() {
                    count += (td.token as usize >> 16) & 0x7FFF;
                }

                async_list.write((&*queuehead as *const QueueHead) as u32 | 2);
                usb_cmd.writef(1 << 5 | 1, true);

                for td in tds.iter().rev() {
                    while volatile_load(td as *const Qtd).token & 1 << 7 == 1 << 7 {
                        context_switch(false);
                    }
                }

                usb_cmd.writef(1 << 5 | 1, false);
                async_list.write(0);
            }
        }

        count
    }
}
