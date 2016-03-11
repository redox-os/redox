use alloc::boxed::Box;

use arch::memory;

use collections::slice;
use collections::string::ToString;
use collections::vec::Vec;
use collections::vec_deque::VecDeque;

use core::ptr;

use common::debug;

use drivers::pci::config::PciConfig;
use drivers::io::{Io, Pio};

use network::common::*;
use network::scheme::*;

use fs::{KScheme, Resource, Url};

use system::error::Result;

use sync::Intex;

const RTL8139_TSR_OWN: u32 = 1 << 13;

const RTL8139_CR_RST: u8 = 1 << 4;
const RTL8139_CR_RE: u8 = 1 << 3;
const RTL8139_CR_TE: u8 = 1 << 2;
const RTL8139_CR_BUFE: u8 = 1 << 0;

const RTL8139_ISR_SERR: u16 = 1 << 15;
const RTL8139_ISR_TIMEOUT: u16 = 1 << 14;
const RTL8139_ISR_LENCHG: u16 = 1 << 13;
const RTL8139_ISR_FOVW: u16 = 1 << 6;
const RTL8139_ISR_PUN_LINKCHG: u16 = 1 << 5;
const RTL8139_ISR_RXOVW: u16 = 1 << 4;
const RTL8139_ISR_TER: u16 = 1 << 3;
const RTL8139_ISR_TOK: u16 = 1 << 2;
const RTL8139_ISR_RER: u16 = 1 << 1;
const RTL8139_ISR_ROK: u16 = 1 << 0;

const RTL8139_TCR_IFG: u32 = 0b11 << 24;

const RTL8139_RCR_WRAP: u32 = 1 << 7;
const RTL8139_RCR_AR: u32 = 1 << 4;
const RTL8139_RCR_AB: u32 = 1 << 3;
const RTL8139_RCR_AM: u32 = 1 << 2;
const RTL8139_RCR_APM: u32 = 1 << 1;

#[repr(packed)]
struct Txd {
    pub address_port: Pio<u32>,
    pub status_port: Pio<u32>,
    pub buffer: usize,
}

pub struct Rtl8139Port {
    pub idr: [Pio<u8>; 6],
    pub rbstart: Pio<u32>,
    pub cr: Pio<u8>,
    pub capr: Pio<u16>,
    pub cbr: Pio<u16>,
    pub imr: Pio<u16>,
    pub isr: Pio<u16>,
    pub tcr: Pio<u32>,
    pub rcr: Pio<u32>,
    pub config1: Pio<u8>,
}

impl Rtl8139Port {
    pub fn new(base: u16) -> Self {
        return Rtl8139Port {
            idr: [Pio::<u8>::new(base + 0x00),
                  Pio::<u8>::new(base + 0x01),
                  Pio::<u8>::new(base + 0x02),
                  Pio::<u8>::new(base + 0x03),
                  Pio::<u8>::new(base + 0x04),
                  Pio::<u8>::new(base + 0x05)],
            rbstart: Pio::<u32>::new(base + 0x30),
            cr: Pio::<u8>::new(base + 0x37),
            capr: Pio::<u16>::new(base + 0x38),
            cbr: Pio::<u16>::new(base + 0x3A),
            imr: Pio::<u16>::new(base + 0x3C),
            isr: Pio::<u16>::new(base + 0x3E),
            tcr: Pio::<u32>::new(base + 0x40),
            rcr: Pio::<u32>::new(base + 0x44),
            config1: Pio::<u8>::new(base + 0x52),
        };
    }
}

pub struct Rtl8139 {
    pci: PciConfig,
    base: usize,
    memory_mapped: bool,
    irq: u8,
    resources: Intex<Vec<*mut NetworkResource>>,
    inbound: VecDeque<Vec<u8>>,
    outbound: VecDeque<Vec<u8>>,
    txds: Vec<Txd>,
    txd_i: usize,
    port: Rtl8139Port,
}

impl Rtl8139 {
    pub fn new(mut pci: PciConfig) -> Box<Self> {
        let pci_id = unsafe { pci.read(0x00) };
        let revision = (unsafe { pci.read(0x08) } & 0xFF) as u8;
        if pci_id == 0x813910EC && revision < 0x20 {
            debugln!("Not an 8139C+ compatible chip")
        }

        let base = unsafe { pci.read(0x10) as usize };
        let irq = unsafe { pci.read(0x3C) as u8 & 0xF };

        let mut module = box Rtl8139 {
            pci: pci,
            base: base & 0xFFFFFFF0,
            memory_mapped: base & 1 == 0,
            irq: irq,
            resources: Intex::new(Vec::new()),
            inbound: VecDeque::new(),
            outbound: VecDeque::new(),
            txds: Vec::new(),
            txd_i: 0,
            port: Rtl8139Port::new((base & 0xFFFFFFF0) as u16),
        };

        unsafe { module.init() };

        module
    }

    unsafe fn init(&mut self) {
        debug::d("RTL8139 on: ");
        debug::dh(self.base);
        if self.memory_mapped {
            debug::d(" memory mapped");
        } else {
            debug::d(" port mapped");
        }
        debug::d(" IRQ: ");
        debug::dbh(self.irq);

        self.pci.flag(4, 4, true); // Bus mastering

        let base = self.base as u16;

        self.port.config1.write(0);
        self.port.cr.write(RTL8139_CR_RST);
        while self.port.cr.read() & RTL8139_CR_RST != 0 {}

        debug::d(" MAC: ");
        MAC_ADDR = MacAddr {
            bytes: [self.port.idr[0].read(),
                    self.port.idr[1].read(),
                    self.port.idr[2].read(),
                    self.port.idr[3].read(),
                    self.port.idr[4].read(),
                    self.port.idr[5].read()],
        };
        debug::d(&MAC_ADDR.to_string());

        let receive_buffer = memory::alloc(10240);
        self.port.rbstart.write(receive_buffer as u32);

        for i in 0..4 {
            self.txds.push(Txd {
                address_port: Pio::<u32>::new(base + 0x20 + (i as u16) * 4),
                status_port: Pio::<u32>::new(base + 0x10 + (i as u16) * 4),
                buffer: memory::alloc(4096),
            });
        }

        self.port.imr.write(RTL8139_ISR_TOK | RTL8139_ISR_ROK);
        debug::d(" IMR: ");
        debug::dh(self.port.imr.read() as usize);

        self.port.cr.write(RTL8139_CR_RE | RTL8139_CR_TE);
        debug::d(" CMD: ");
        debug::dbh(self.port.cr.read());

        self.port.rcr.write(RTL8139_RCR_WRAP | RTL8139_RCR_AR | RTL8139_RCR_AB | RTL8139_RCR_AM |
                            RTL8139_RCR_APM);
        debug::d(" RCR: ");
        debug::dh(self.port.rcr.read() as usize);

        self.port.tcr.writef(RTL8139_TCR_IFG, true);
        debug::d(" TCR: ");
        debug::dh(self.port.tcr.read() as usize);

        debug::dl();
    }

    unsafe fn receive_inbound(&mut self) {
        let receive_buffer = self.port.rbstart.read() as usize;
        let mut capr = (self.port.capr.read() + 16) as usize;
        let cbr = self.port.cbr.read() as usize;

        while capr != cbr {
            let frame_addr = receive_buffer + capr + 4;
            let frame_status = ptr::read((receive_buffer + capr) as *const u16) as usize;
            let frame_len = ptr::read((receive_buffer + capr + 2) as *const u16) as usize;

            debug::d("Recv ");
            debug::dh(capr as usize);
            debug::d(" ");
            debug::dh(frame_status);
            debug::d(" ");
            debug::dh(frame_addr);
            debug::d(" ");
            debug::dh(frame_len);
            debug::dl();

            self.inbound.push_back(Vec::from(slice::from_raw_parts(frame_addr as *const u8, frame_len - 4)));

            capr = capr + frame_len + 4;
            capr = (capr + 3) & (0xFFFFFFFF - 3);
            if capr >= 8192 {
                capr -= 8192
            }

            self.port.capr.write((capr as u16) - 16);
        }
    }

    unsafe fn send_outbound(&mut self) {
        while let Some(bytes) = self.outbound.pop_front() {
            if let Some(ref mut txd) = self.txds.get_mut(self.txd_i) {
                if bytes.len() < 4096 {
                    while !txd.status_port.readf(RTL8139_TSR_OWN) {}

                    debug::d("Send ");
                    debug::dh(self.txd_i as usize);
                    debug::d(" ");
                    debug::dh(txd.status_port.read() as usize);
                    debug::d(" ");
                    debug::dh(txd.buffer);
                    debug::d(" ");
                    debug::dh(bytes.len() & 0xFFF);
                    debug::dl();

                    ::memcpy(txd.buffer as *mut u8, bytes.as_ptr(), bytes.len());

                    txd.address_port.write(txd.buffer as u32);
                    txd.status_port.write(bytes.len() as u32 & 0xFFF);

                    self.txd_i = (self.txd_i + 1) % 4;
                } else {
                    debug::dl();
                    debug::d("RTL8139: Frame too long for transmit: ");
                    debug::dd(bytes.len());
                    debug::dl();
                }
            } else {
                debug::d("RTL8139: TXD Overflow!\n");
                self.txd_i = 0;
            }
        }
    }
}

impl KScheme for Rtl8139 {
    fn scheme(&self) -> &str {
        "network"
    }

    fn open(&mut self, _: Url, _: usize) -> Result<Box<Resource>> {
        Ok(NetworkResource::new(self))
    }

    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            let isr = self.port.isr.read();
            self.port.isr.write(isr);

            // dh(isr as usize);
            // dl();

            self.sync();
        }
    }
}

impl NetworkScheme for Rtl8139 {
    fn add(&mut self, resource: *mut NetworkResource) {
        self.resources.lock().push(resource);
    }

    fn remove(&mut self, resource: *mut NetworkResource) {
        let mut resources = self.resources.lock();

        let mut i = 0;
        while i < resources.len() {
            let mut remove = false;

            match resources.get(i) {
                Some(ptr) => if *ptr == resource {
                    remove = true;
                } else {
                    i += 1;
                },
                None => break,
            }

            if remove {
                resources.remove(i);
            }
        }
    }

    fn sync(&mut self) {
        unsafe {
            {
                let resources = self.resources.lock();

                for resource in resources.iter() {
                    while let Some(bytes) = (**resource).outbound.lock().pop_front() {
                        self.outbound.push_back(bytes);
                    }
                }
            }

            self.send_outbound();

            self.receive_inbound();

            {
                let resources = self.resources.lock();

                while let Some(bytes) = self.inbound.pop_front() {
                    for resource in resources.iter() {
                        (**resource).inbound.lock().push_back(bytes.clone());
                    }
                }
            }
        }
    }
}
