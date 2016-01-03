use alloc::boxed::Box;

use collections::vec::Vec;

use core::intrinsics::{volatile_load, volatile_store};
use core::mem::size_of;
use core::ptr::{self, read, write};

use common::debug;
use common::memory;
use common::time::{self, Duration};

use drivers::pciconfig::PciConfig;

use schemes::KScheme;

use super::hci::{UsbHci, UsbMsg};
use super::setup::Setup;

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
                let CAPLENGTH = self.base as *mut u8;

                let opbase = self.base + read(CAPLENGTH) as usize;

                let USBSTS = (opbase + 4) as *mut u32;
                // debug::d(" USBSTS ");
                // debug::dh(*USBSTS as usize);

                write(USBSTS, 0b111111);

                // debug::d(" USBSTS ");
                // debug::dh(*USBSTS as usize);

                // let FRINDEX = (opbase + 0xC) as *mut u32;
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

        let CAPLENGTH = self.base as *mut u8;
        let HCSPARAMS = (self.base + 4) as *mut u32;
        let HCCPARAMS = (self.base + 8) as *mut u32;

        debug::d(" CAPLENGTH ");
        debug::dd(read(CAPLENGTH) as usize);

        debug::d(" HCSPARAMS ");
        debug::dh(read(HCSPARAMS) as usize);

        debug::d(" HCCPARAMS ");
        debug::dh(read(HCCPARAMS) as usize);

        let ports = (read(HCSPARAMS) & 0b1111) as usize;
        debug::d(" PORTS ");
        debug::dd(ports);

        let eecp = ((read(HCCPARAMS) >> 8) & 0xFF) as u8;
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

        if read(USBSTS) & (1 << 12) == 0 {
            debug::d("Halting");
            debug::d(" CMD ");
            debug::dh(read(USBCMD) as usize);

            debug::d(" STS ");
            debug::dh(read(USBSTS) as usize);

            write(USBCMD, read(USBCMD) & 0xFFFFFFF0);

            debug::d(" CMD ");
            debug::dh(*USBCMD as usize);

            debug::d(" STS ");
            debug::dh(*USBSTS as usize);
            debug::dl();

            debug::d("Waiting");
            while volatile_load(USBSTS) & (1 << 12) != (1 << 12) {}

            debug::d(" CMD ");
            debug::dh(read(USBCMD) as usize);

            debug::d(" STS ");
            debug::dh(read(USBSTS) as usize);
            debug::dl();
        }

        debug::d("Resetting");
        debug::d(" CMD ");
        debug::dh(read(USBCMD) as usize);

        debug::d(" STS ");
        debug::dh(read(USBSTS) as usize);

        write(USBCMD, read(USBCMD) | (1 << 1));

        debug::d(" CMD ");
        debug::dh(read(USBCMD) as usize);

        debug::d(" STS ");
        debug::dh(read(USBSTS) as usize);
        debug::dl();

        debug::d("Waiting");
        while volatile_load(USBCMD) & 1 << 1 == 1 << 1 {}

        debug::d(" CMD ");
        debug::dh(read(USBCMD) as usize);

        debug::d(" STS ");
        debug::dh(read(USBSTS) as usize);
        debug::dl();

        debug::d("Enabling");
        debug::d(" CMD ");
        debug::dh(read(USBCMD) as usize);

        debug::d(" STS ");
        debug::dh(read(USBSTS) as usize);

        write(USBINTR, 0b111111);

        write(USBCMD, read(USBCMD) | 1);
        write(CONFIGFLAG, 1);

        debug::d(" CMD ");
        debug::dh(read(USBCMD) as usize);

        debug::d(" STS ");
        debug::dh(read(USBSTS) as usize);
        debug::dl();

        debug::d("Waiting");
        while volatile_load(USBSTS) & 1 << 12 == 1 << 12 {}

        debug::d(" CMD ");
        debug::dh(read(USBCMD) as usize);

        debug::d(" STS ");
        debug::dh(read(USBSTS) as usize);
        debug::dl();

        for i in 0..ports as isize {
            debug::dd(i as usize);
            debug::d(": ");
            debug::dh(read(PORTSC.offset(i)) as usize);
            debug::dl();

            if read(PORTSC.offset(i)) & 1 == 1 {
                debug::d("Device on port ");
                debug::dd(i as usize);
                debug::d(" ");
                debug::dh(read(PORTSC.offset(i)) as usize);
                debug::dl();

                if read(PORTSC.offset(i)) & 1 << 1 == 1 << 1 {
                    debug::d("Connection Change");
                    debug::d(" ");
                    debug::dh(read(PORTSC.offset(i)) as usize);

                    write(PORTSC.offset(i), read(PORTSC.offset(i)) | (1 << 1));

                    debug::d(" ");
                    debug::dh(read(PORTSC.offset(i)) as usize);
                    debug::dl();
                }

                if read(PORTSC.offset(i)) & 1 << 2 == 0 {
                    debug::d("Reset");
                    debug::d(" ");
                    debug::dh(read(PORTSC.offset(i)) as usize);

                    write(PORTSC.offset(i), read(PORTSC.offset(i)) | (1 << 8));

                    debug::d(" ");
                    debug::dh(read(PORTSC.offset(i)) as usize);

                    write(PORTSC.offset(i),
                    read(PORTSC.offset(i)) & 0xFFFFFEFF);

                    debug::d(" ");
                    debug::dh(read(PORTSC.offset(i)) as usize);
                    debug::dl();

                    debug::d("Wait");
                    debug::d(" ");
                    debug::dh(read(PORTSC.offset(i)) as usize);

                    while volatile_load(PORTSC.offset(i)) & 1 << 8 == 1 << 8 {
                        volatile_store(PORTSC.offset(i), volatile_load(PORTSC.offset(i)) & 0xFFFFFEFF);
                    }

                    debug::d(" ");
                    debug::dh(read(PORTSC.offset(i)) as usize);
                    debug::dl();
                }

                debug::d("Port Enabled ");
                debug::dh(read(PORTSC.offset(i)) as usize);
                debug::dl();

                self.device(i as u8 + 1);
            }
        }
    }
}

impl UsbHci for Ehci {
    fn msg(&mut self, address: u8, endpoint: u8, msgs: &[UsbMsg]) -> usize {
        let mut tds = Vec::new();
        for msg in msgs.iter().rev() {
            let link_ptr = match tds.last() {
                Some(td) => (td as *const Qtd) as u32,
                None => 1
            };

            match *msg {
                UsbMsg::Setup(setup) => tds.push(Qtd {
                    next: link_ptr,
                    next_alt: 1,
                    token: (size_of::<Setup>() as u32) << 16 | 0b10 << 8 | 1 << 7,
                    buffers: [(setup as *const Setup) as u32, 0, 0, 0, 0]
                }),
                UsbMsg::In(ref data) => tds.push(Qtd {
                    next: link_ptr,
                    next_alt: 1,
                    token: ((data.len() as u32) & 0x7FFF) << 16 | 0b01 << 8 | 1 << 7,
                    buffers: [data.as_ptr() as u32, 0, 0, 0, 0]
                }),
                UsbMsg::InIso(ref data) => tds.push(Qtd {
                    next: link_ptr,
                    next_alt: 1,
                    token: ((data.len() as u32) & 0x7FFF) << 16 | 0b01 << 8 | 1 << 7,
                    buffers: [data.as_ptr() as u32, 0, 0, 0, 0]
                }),
                UsbMsg::Out(ref data) => tds.push(Qtd {
                    next: link_ptr,
                    next_alt: 1,
                    token: ((data.len() as u32) & 0x7FFF) << 16 | 0b00 << 8 | 1 << 7,
                    buffers: [data.as_ptr() as u32, 0, 0, 0, 0]
                }),
                UsbMsg::OutIso(ref data) => tds.push(Qtd {
                    next: link_ptr,
                    next_alt: 1,
                    token: ((data.len() as u32) & 0x7FFF) << 16 | 0b00 << 8 | 1 << 7,
                    buffers: [data.as_ptr() as u32, 0, 0, 0, 0]
                })
            }
        }

        let mut count = 0;

        if ! tds.is_empty() {
            unsafe {
                let CAPLENGTH = self.base as *mut u8;

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

                let queuehead = box QueueHead {
                    next: 1,
                    characteristics: 1024 << 16 | 1 << 15 | 1 << 14 | 0b10 << 12 | (endpoint as u32) << 8 | address as u32,
                    capabilities: 0b01 << 30,
                    qtd_ptr: (tds.last().unwrap() as *const Qtd) as u32,
                    qtd: *tds.last().unwrap()
                };

                debug::d("Prepare");
                debug::d(" CMD ");
                debug::dh(*USBCMD as usize);

                debug::d(" PTR ");
                debug::dh((&*queuehead as *const QueueHead) as usize);
                debug::dl();

                debug::d("Send");
                debug::d(" CMD ");
                debug::dh(*USBCMD as usize);

                debug::d(" STS ");
                debug::dh(*USBSTS as usize);

                volatile_store(ASYNCLISTADDR, (&*queuehead as *const QueueHead) as u32 | 2);
                volatile_store(USBCMD, volatile_load(USBCMD) | 1 << 5);

                debug::d(" CMD ");
                debug::dh(*USBCMD as usize);

                debug::d(" STS ");
                debug::dh(*USBSTS as usize);

                volatile_store(USBCMD, volatile_load(USBCMD) | 1);

                debug::d(" CMD ");
                debug::dh(*USBCMD as usize);

                debug::d(" STS ");
                debug::dh(*USBSTS as usize);
                debug::dl();

                debug::d("Wait");
                debug::d(" CMD ");
                debug::dh(*USBCMD as usize);

                debug::d(" STS ");
                debug::dh(*USBSTS as usize);
                debug::dl();

                while volatile_load(USBSTS) & 0xA000 == 0xA000 {}

                debug::d("Stop");
                debug::d(" CMD ");
                debug::dh(*USBCMD as usize);

                debug::d(" STS ");
                debug::dh(*USBSTS as usize);

                volatile_store(USBCMD, volatile_load(USBCMD) & 0xFFFFFFFF - 1 << 5);
                volatile_store(ASYNCLISTADDR, 0);

                debug::d(" CMD ");
                debug::dh(*USBCMD as usize);

                debug::d(" STS ");
                debug::dh(*USBSTS as usize);
                debug::dl();
            }
        }

        count
    }
}
