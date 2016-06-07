use alloc::boxed::Box;

use arch::memory;

use collections::slice;
use collections::vec::Vec;
use collections::vec_deque::VecDeque;

use core::ptr;

use drivers::pci::config::PciConfig;
use drivers::io::{Io, Pio};

use network::common::*;
use network::scheme::*;

use fs::{KScheme, Resource, Url};

use system::error::Result;

use sync::Intex;

bitflags! {
    flags TsrFlags: u32 {
        const TSR_OWN = 1 << 13
    }
}

bitflags! {
    flags CrFlags: u8 {
        const CR_RST = 1 << 4,
        const CR_RE = 1 << 3,
        const CR_TE = 1 << 2,
        const CR_BUFE = 1 << 0
    }
}

bitflags! {
    flags IsrFlags: u16 {
        const ISR_SERR = 1 << 15,
        const ISR_TIMEOUT = 1 << 14,
        const ISR_LENCHG = 1 << 13,
        const ISR_FOVW = 1 << 6,
        const ISR_PUN_LINKCHG = 1 << 5,
        const ISR_RXOVW = 1 << 4,
        const ISR_TER = 1 << 3,
        const ISR_TOK = 1 << 2,
        const ISR_RER = 1 << 1,
        const ISR_ROK = 1 << 0
    }
}

bitflags! {
    flags TcrFlags: u32 {
        const TCR_IFG = 0b11 << 24
    }
}

bitflags! {
    flags RcrFlags: u32 {
        const RCR_WRAP = 1 << 7,
        const RCR_AR = 1 << 4,
        const RCR_AB = 1 << 3,
        const RCR_AM = 1 << 2,
        const RCR_APM = 1 << 1
    }
}

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
            debugln!("Not an 8139C+ compatible chip");
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
        debugln!(" + RTL8139 on: {:X}, IRQ: {:X}", self.base, self.irq);

        self.pci.flag(4, 4, true); // Bus mastering

        let base = self.base as u16;

        self.port.config1.write(0);
        self.port.cr.write(CR_RST.bits);
        while self.port.cr.read() & CR_RST.bits != 0 {}

        MAC_ADDR = MacAddr {
            bytes: [self.port.idr[0].read(),
                    self.port.idr[1].read(),
                    self.port.idr[2].read(),
                    self.port.idr[3].read(),
                    self.port.idr[4].read(),
                    self.port.idr[5].read()],
        };
        debugln!("   - MAC: {}", &MAC_ADDR.to_string());

        let receive_buffer = memory::alloc(10240);
        self.port.rbstart.write(receive_buffer as u32);

        for i in 0..4 {
            self.txds.push(Txd {
                address_port: Pio::<u32>::new(base + 0x20 + (i as u16) * 4),
                status_port: Pio::<u32>::new(base + 0x10 + (i as u16) * 4),
                buffer: memory::alloc(4096),
            });
        }

        self.port.imr.write((ISR_TOK | ISR_ROK).bits);
        self.port.cr.write((CR_RE | CR_TE).bits);
        self.port.rcr.write((RCR_WRAP | RCR_AR | RCR_AB | RCR_AM | RCR_APM).bits);
        self.port.tcr.writef(TCR_IFG.bits, true);
    }

    unsafe fn receive_inbound(&mut self) {
        let receive_buffer = self.port.rbstart.read() as usize;
        let mut capr = (self.port.capr.read() + 16) as usize;
        let cbr = self.port.cbr.read() as usize;

        while capr != cbr {
            let frame_addr = receive_buffer + capr + 4;
            // let frame_status = ptr::read((receive_buffer + capr) as *const u16) as usize;
            let frame_len = ptr::read((receive_buffer + capr + 2) as *const u16) as usize;

            self.inbound.push_back(Vec::from(slice::from_raw_parts(frame_addr as *const u8,
                                                                   frame_len - 4)));

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
                    while !txd.status_port.readf(TSR_OWN.bits) {}

                    ::memcpy(txd.buffer as *mut u8, bytes.as_ptr(), bytes.len());

                    txd.address_port.write(txd.buffer as u32);
                    txd.status_port.write(bytes.len() as u32 & 0xFFF);

                    self.txd_i = (self.txd_i + 1) % 4;
                } else {
                    debugln!("RTL8139: Frame too long for transmit: {}", bytes.len());
                }
            } else {
                debugln!("RTL8139: TXD Overflow!");
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
                Some(ptr) => {
                    if *ptr == resource {
                        remove = true;
                    } else {
                        i += 1;
                    }
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
                        (**resource).inbound.send(bytes.clone());
                    }
                }
            }
        }
    }
}
