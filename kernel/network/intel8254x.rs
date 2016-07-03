use alloc::boxed::Box;

use arch::memory;

use collections::slice;
use collections::vec::Vec;
use collections::vec_deque::VecDeque;

use core::cell::UnsafeCell;
use core::ptr;

use drivers::pci::config::PciConfig;

use network::common::*;
use network::scheme::*;

use fs::{KScheme, Resource, Url};

use system::error::Result;

const CTRL: u32 = 0x00;
const CTRL_LRST: u32 = 1 << 3;
const CTRL_ASDE: u32 = 1 << 5;
const CTRL_SLU: u32 = 1 << 6;
const CTRL_ILOS: u32 = 1 << 7;
const CTRL_VME: u32 = 1 << 30;
const CTRL_PHY_RST: u32 = 1 << 31;

const STATUS: u32 = 0x08;

const FCAL: u32 = 0x28;
const FCAH: u32 = 0x2C;
const FCT: u32 = 0x30;
const FCTTV: u32 = 0x170;

const ICR: u32 = 0xC0;

const IMS: u32 = 0xD0;
const IMS_TXDW: u32 = 1;
const IMS_TXQE: u32 = 1 << 1;
const IMS_LSC: u32 = 1 << 2;
const IMS_RXSEQ: u32 = 1 << 3;
const IMS_RXDMT: u32 = 1 << 4;
const IMS_RX: u32 = 1 << 6;
const IMS_RXT: u32 = 1 << 7;

const RCTL: u32 = 0x100;
const RCTL_EN: u32 = 1 << 1;
const RCTL_UPE: u32 = 1 << 3;
const RCTL_MPE: u32 = 1 << 4;
const RCTL_LPE: u32 = 1 << 5;
const RCTL_LBM: u32 = 1 << 6 | 1 << 7;
const RCTL_BAM: u32 = 1 << 15;
const RCTL_BSIZE1: u32 = 1 << 16;
const RCTL_BSIZE2: u32 = 1 << 17;
const RCTL_BSEX: u32 = 1 << 25;
const RCTL_SECRC: u32 = 1 << 26;

const RDBAL: u32 = 0x2800;
const RDBAH: u32 = 0x2804;
const RDLEN: u32 = 0x2808;
const RDH: u32 = 0x2810;
const RDT: u32 = 0x2818;

const RAL0: u32 = 0x5400;
const RAH0: u32 = 0x5404;

#[repr(packed)]
struct Rd {
    buffer: u64,
    length: u16,
    checksum: u16,
    status: u8,
    error: u8,
    special: u16,
}
const RD_DD: u8 = 1;
const RD_EOP: u8 = 1 << 1;

const TCTL: u32 = 0x400;
const TCTL_EN: u32 = 1 << 1;
const TCTL_PSP: u32 = 1 << 3;

const TDBAL: u32 = 0x3800;
const TDBAH: u32 = 0x3804;
const TDLEN: u32 = 0x3808;
const TDH: u32 = 0x3810;
const TDT: u32 = 0x3818;

#[repr(packed)]
struct Td {
    buffer: u64,
    length: u16,
    cso: u8,
    command: u8,
    status: u8,
    css: u8,
    special: u16,
}
const TD_CMD_EOP: u8 = 1;
const TD_CMD_IFCS: u8 = 1 << 1;
const TD_CMD_RS: u8 = 1 << 3;
const TD_DD: u8 = 1;

pub struct Intel8254x {
    pub pci: PciConfig,
    pub base: usize,
    pub memory_mapped: bool,
    pub irq: u8,
    pub resources: UnsafeCell<Vec<*mut NetworkResource>>,
    pub inbound: VecDeque<Vec<u8>>,
    pub outbound: VecDeque<Vec<u8>>,
}

impl KScheme for Intel8254x {
    fn scheme(&self) -> &str {
        "network"
    }

    fn open(&mut self, _: Url, _: usize) -> Result<Box<Resource>> {
        Ok(NetworkResource::new(self))
    }

    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            unsafe { self.read(ICR) };

            self.sync();
        }
    }
}

impl NetworkScheme for Intel8254x {
    fn add(&mut self, resource: *mut NetworkResource) {
        unsafe { &mut *self.resources.get() }.push(resource);
    }

    fn remove(&mut self, resource: *mut NetworkResource) {
        let mut resources = unsafe { &mut *self.resources.get() };

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
        {
            let resources = unsafe { &mut *self.resources.get() };

            for resource in resources.iter() {
                while let Some(bytes) = unsafe { &mut *(**resource).outbound.get() }.pop_front() {
                    self.outbound.push_back(bytes);
                }
            }
        }

        unsafe { self.send_outbound(); }

        unsafe { self.receive_inbound(); }

        {
            let resources = unsafe { &mut *self.resources.get() };

            while let Some(bytes) = self.inbound.pop_front() {
                for resource in resources.iter() {
                    unsafe { (**resource).inbound.send(bytes.clone(), "Intel8254x::sync") };
                }
            }
        }
    }
}

impl Intel8254x {
    pub unsafe fn new(mut pci: PciConfig) -> Box<Self> {
        let base = pci.read(0x10) as usize;

        let mut module = box Intel8254x {
            pci: pci,
            base: base & 0xFFFFFFF0,
            memory_mapped: base & 1 == 0,
            irq: pci.read(0x3C) as u8 & 0xF,
            resources: UnsafeCell::new(Vec::new()),
            inbound: VecDeque::new(),
            outbound: VecDeque::new(),
        };

        module.init();

        module
    }

    pub unsafe fn receive_inbound(&mut self) {
        let receive_ring = self.read(RDBAL) as *mut Rd;
        let length = self.read(RDLEN);

        for tail in 0..length / 16 {
            let rd = &mut *receive_ring.offset(tail as isize);
            if rd.status & RD_DD == RD_DD {
                self.inbound.push_back(Vec::from(slice::from_raw_parts(rd.buffer as *const u8, rd.length as usize)));

                rd.status = 0;
            }
        }
    }

    pub unsafe fn send_outbound(&mut self) {
        while let Some(bytes) = self.outbound.pop_front() {
            let transmit_ring = self.read(TDBAL) as *mut Td;
            let length = self.read(TDLEN);

            loop {
                let head = self.read(TDH);
                let mut tail = self.read(TDT);
                let old_tail = tail;

                tail += 1;
                if tail >= length / 16 {
                    tail = 0;
                }

                if tail != head {
                    if bytes.len() < 16384 {
                        let td = &mut *transmit_ring.offset(old_tail as isize);

                        ::memcpy(td.buffer as *mut u8, bytes.as_ptr(), bytes.len());
                        td.length = (bytes.len() & 0x3FFF) as u16;
                        td.cso = 0;
                        td.command = TD_CMD_EOP | TD_CMD_IFCS | TD_CMD_RS;
                        td.status = 0;
                        td.css = 0;
                        td.special = 0;

                        self.write(TDT, tail);
                    } else {
                        // TODO: More than one TD
                        debugln!("Intel 8254x: Frame too long for transmit: {}", bytes.len());
                    }

                    break;
                }
            }
        }
    }

    pub unsafe fn read(&self, register: u32) -> u32 {
        if self.memory_mapped {
            ptr::read((self.base + register as usize) as *mut u32)
        } else {
            0
        }
    }

    pub unsafe fn write(&self, register: u32, data: u32) -> u32 {
        if self.memory_mapped {
            ptr::write((self.base + register as usize) as *mut u32, data);
            ptr::read((self.base + register as usize) as *mut u32)
        } else {
            0
        }
    }

    pub unsafe fn flag(&self, register: u32, flag: u32, value: bool) {
        if value {
            self.write(register, self.read(register) | flag);
        } else {
            self.write(register, self.read(register) & (0xFFFFFFFF - flag));
        }
    }

    pub unsafe fn init(&mut self) {
        syslog_info!(" + Intel 8254x on: {:X}, IRQ: {:X}", self.base, self.irq);

        self.pci.flag(4, 4, true); // Bus mastering

        // Enable auto negotiate, link, clear reset, do not Invert Loss-Of Signal
        self.flag(CTRL, CTRL_ASDE | CTRL_SLU, true);
        self.flag(CTRL, CTRL_LRST, false);
        self.flag(CTRL, CTRL_PHY_RST, false);
        self.flag(CTRL, CTRL_ILOS, false);

        // No flow control
        self.write(FCAH, 0);
        self.write(FCAL, 0);
        self.write(FCT, 0);
        self.write(FCTTV, 0);

        // Do not use VLANs
        self.flag(CTRL, CTRL_VME, false);

        // TODO: Clear statistical counters

        let mac_low = self.read(RAL0);
        let mac_high = self.read(RAH0);
        MAC_ADDR = MacAddr {
            bytes: [mac_low as u8,
                    (mac_low >> 8) as u8,
                    (mac_low >> 16) as u8,
                    (mac_low >> 24) as u8,
                    mac_high as u8,
                    (mac_high >> 8) as u8],
        };
        syslog_info!("   - MAC: {}", &MAC_ADDR.to_string());

        //
        // MTA => 0;
        //

        // Receive Buffer
        let receive_ring_length = 1024;
        let receive_ring = memory::alloc(receive_ring_length * 16) as *mut Rd;
        for i in 0..receive_ring_length {
            let receive_buffer = memory::alloc(16384);
            ptr::write(receive_ring.offset(i as isize),
                       Rd {
                           buffer: receive_buffer as u64,
                           length: 0,
                           checksum: 0,
                           status: 0,
                           error: 0,
                           special: 0,
                       });
        }

        self.write(RDBAH, 0);
        self.write(RDBAL, receive_ring as u32);
        self.write(RDLEN, (receive_ring_length * 16) as u32);
        self.write(RDH, 0);
        self.write(RDT, receive_ring_length as u32 - 1);

        // Transmit Buffer
        let transmit_ring_length = 64;
        let transmit_ring = memory::alloc(transmit_ring_length * 16) as *mut Td;
        for i in 0..transmit_ring_length {
            let transmit_buffer = memory::alloc(16384);
            ptr::write(transmit_ring.offset(i as isize),
                       Td {
                           buffer: transmit_buffer as u64,
                           length: 0,
                           cso: 0,
                           command: 0,
                           status: 0,
                           css: 0,
                           special: 0,
                       });
        }

        self.write(TDBAH, 0);
        self.write(TDBAL, transmit_ring as u32);
        self.write(TDLEN, (transmit_ring_length * 16) as u32);
        self.write(TDH, 0);
        self.write(TDT, 0);

        self.write(IMS, IMS_RXT | IMS_RX | IMS_RXDMT | IMS_RXSEQ | IMS_LSC | IMS_TXQE | IMS_TXDW);

        self.flag(RCTL, RCTL_EN, true);
        self.flag(RCTL, RCTL_UPE, true);
        // self.flag(RCTL, RCTL_MPE, true);
        self.flag(RCTL, RCTL_LPE, true);
        self.flag(RCTL, RCTL_LBM, false);
        // RCTL.RDMTS = Minimum threshold size ???
        // RCTL.MO = Multicast offset
        self.flag(RCTL, RCTL_BAM, true);
        self.flag(RCTL, RCTL_BSIZE1, true);
        self.flag(RCTL, RCTL_BSIZE2, false);
        self.flag(RCTL, RCTL_BSEX, true);
        self.flag(RCTL, RCTL_SECRC, true);

        self.flag(TCTL, TCTL_EN, true);
        self.flag(TCTL, TCTL_PSP, true);
        // TCTL.CT = Collition threshold
        // TCTL.COLD = Collision distance
        // TIPG Packet Gap
        // TODO ...
    }
}
