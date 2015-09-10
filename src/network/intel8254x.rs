use core::ops::{DerefMut,Drop};

use common::memory::*;
use common::pci::*;
use common::pio::*;
use common::scheduler::*;

use network::common::*;
use network::ethernet::*;

use programs::common::*;

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

struct RD {
    buffer: u64,
    length: u16,
    checksum: u16,
    status: u8,
    error: u8,
    special: u16
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

struct TD {
    buffer: u64,
    length: u16,
    cso: u8,
    command: u8,
    status: u8,
    css: u8,
    special: u16
}
    const TD_CMD_EOP: u8 = 1;
    const TD_CMD_IFCS: u8 = 1 << 1;
    const TD_CMD_RS: u8 = 1 << 3;
    const TD_DD: u8 = 1;

pub struct Intel8254xResource {
    pub nic: *mut Intel8254x,
    pub ptr: *mut Intel8254xResource,
    pub inbound: Queue<Vec<u8>>,
    pub outbound: Queue<Vec<u8>>
}

impl Intel8254xResource {
    pub fn new(nic: &mut Intel8254x) -> Box<Intel8254xResource> {
        let mut ret = box Intel8254xResource {
            nic: nic,
            ptr: 0 as *mut Intel8254xResource,
            inbound: Queue::new(),
            outbound: Queue::new()
        };

        unsafe{
            ret.ptr = ret.deref_mut();

            if ret.nic as usize > 0 && ret.ptr as usize > 0 {
                let reenable = start_no_ints();

                (*ret.nic).resources.push(ret.ptr);

                end_no_ints(reenable);
            }
        }

        return ret;
    }
}

impl Resource for Intel8254xResource {
    fn url(&self) -> URL {
        return URL::from_string(&"network://".to_string());
    }

    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        d("TODO: Implement read for Intel8254x\n");
        return Option::None;
    }

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        loop {
            let option;
            unsafe{
                if self.nic as usize > 0 {
                    (*self.nic).receive_inbound();
                }

                let reenable = start_no_ints();
                option = (*self.ptr).inbound.pop();
                end_no_ints(reenable);
            }

            if let Option::Some(bytes) = option {
                vec.push_all(&bytes);
                return Option::Some(bytes.len());
            }

            sys_yield();
        }
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        unsafe{
            let reenable = start_no_ints();
            (*self.ptr).outbound.push(Vec::from_raw_buf(buf.as_ptr(), buf.len()));
            end_no_ints(reenable);

            if self.nic as usize > 0 {
                (*self.nic).send_outbound();
            }
        }

        return Option::Some(buf.len());
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None;
    }

    fn flush(&mut self) -> bool {
        loop {
            let len;
            unsafe{
                let reenable = start_no_ints();
                len = (*self.ptr).outbound.len();
                end_no_ints(reenable);
            }

            if len == 0 {
                return true;
            }else if self.nic as usize > 0 {
                unsafe {
                    (*self.nic).send_outbound();
                }
            }

            sys_yield();
        }
    }
}

impl Drop for Intel8254xResource {
    fn drop(&mut self){
        if self.nic as usize > 0 {
            unsafe {
                let reenable = start_no_ints();

                let mut i = 0;
                while i < (*self.nic).resources.len() {
                    let mut remove = false;

                    match (*self.nic).resources.get(i) {
                        Option::Some(ptr) => if *ptr == self.ptr {
                            remove = true;
                        }else{
                            i += 1;
                        },
                        Option::None => break
                    }

                    if remove {
                        (*self.nic).resources.remove(i);
                    }
                }

                end_no_ints(reenable);
            }
        }
    }
}

pub struct Intel8254x {
    pub bus: usize,
    pub slot: usize,
    pub func: usize,
    pub base: usize,
    pub memory_mapped: bool,
    pub irq: u8,
    pub resources: Vec<*mut Intel8254xResource>
}

impl SessionItem for Intel8254x {
    fn scheme(&self) -> String {
        return "network".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        return Intel8254xResource::new(self);
    }

    fn on_irq(&mut self, irq: u8){
        if irq == self.irq {
            unsafe{
                let icr = self.read(ICR);
                dh(icr as usize);
                dl();

                self.receive_inbound();
                self.send_outbound();
            }
        }
    }
}

impl Intel8254x {
    pub unsafe fn receive_inbound(&mut self) {
        let reenable = start_no_ints();

        let receive_ring = self.read(RDBAL) as *mut RD;
        let length = self.read(RDLEN);

        for tail in 0..length/16 {
            let rd = &mut *receive_ring.offset(tail as isize);
            if rd.status & RD_DD == RD_DD {
                for resource in self.resources.iter() {
                    (**resource).inbound.push(Vec::from_raw_buf(rd.buffer as *const u8, rd.length as usize));
                }

                rd.status = 0;
            }
        }

        end_no_ints(reenable);
    }

    pub unsafe fn send_outbound(&mut self) {
        let reenable = start_no_ints();

        let mut has_outbound = false;
        for resource in self.resources.iter() {
            if (**resource).outbound.len() > 0 {
                has_outbound = true;
            }
        }

        if has_outbound {
            let transmit_ring = self.read(TDBAL) as *mut TD;
            let length = self.read(TDLEN);
            let head = self.read(TDH);
            let mut tail = self.read(TDT);
            let original_tail = tail;

            loop {
                let old_tail = tail;

                tail += 1;
                if tail >= length / 16 {
                    tail = 0;
                }

                if tail == head {
                    break;
                }

                let mut found = false;

                for resource in self.resources.iter() {
                    if ! found {
                        match (**resource).outbound.pop() {
                            Option::Some(bytes) => {
                                if bytes.len() < 16384 {
                                    found = true;

                                    let td = &mut *transmit_ring.offset(old_tail as isize);
                                    ::memcpy(td.buffer as *mut u8, bytes.as_ptr(), bytes.len());
                                    td.length = bytes.len() as u16;
                                    td.cso = 0;
                                    td.command = TD_CMD_EOP | TD_CMD_IFCS | TD_CMD_RS;
                                    td.status = 0;
                                    td.css = 0;
                                    td.special = 0;
                                }else{
                                    //TODO: More than one TD
                                    dl();
                                    d("Intel 8254x: Frame too long for transmit: ");
                                    dd(bytes.len());
                                    dl();
                                }
                            },
                            Option::None => ()
                        }
                    }
                }

                if ! found {
                    break;
                }
            }

            if tail != original_tail {
                self.write(TDT, tail);
            }
        }

        end_no_ints(reenable);
    }

    pub unsafe fn read(&self, register: u32) -> u32 {
        if self.memory_mapped {
            return ptr::read((self.base + register as usize) as *mut u32);
        }else{
            return 0;
        }
    }

    pub unsafe fn write(&self, register: u32, data: u32) -> u32 {
        if self.memory_mapped {
            ptr::write((self.base + register as usize) as *mut u32, data);
            return ptr::read((self.base + register as usize) as *mut u32);
        }else{
            return 0;
        }
    }

    pub unsafe fn flag(&self, register: u32, flag: u32, value: bool){
        if value {
            self.write(register, self.read(register) | flag);
        }else{
            self.write(register, self.read(register) & (0xFFFFFFFF - flag));
        }
    }

    pub unsafe fn init(&self){
        d("Intel 8254x on: ");
        dh(self.base);
        if self.memory_mapped {
            d(" memory mapped");
        }else{
            d(" port mapped");
        }
        d(", IRQ: ");
        dbh(self.irq);
        dl();

        pci_write(self.bus, self.slot, self.func, 0x04, pci_read(self.bus, self.slot, self.func, 0x04) | (1 << 2)); // Bus mastering

        //Enable auto negotiate, link, clear reset, do not Invert Loss-Of Signal
        self.flag(CTRL, CTRL_ASDE | CTRL_SLU, true);
        self.flag(CTRL, CTRL_LRST, false);
        self.flag(CTRL, CTRL_PHY_RST, false);
        self.flag(CTRL, CTRL_ILOS, false);

        //No flow control
        self.write(FCAH, 0);
        self.write(FCAL, 0);
        self.write(FCT, 0);
        self.write(FCTTV, 0);

        //Do not use VLANs
        self.flag(CTRL, CTRL_VME, false);

        // TODO: Clear statistical counters

        self.write(RAL0, 0x12005452);
        self.write(RAH0, 0x5634);
        /*
        MTA => 0;
        */

        //Receive Buffer
        let receive_ring_length = 1024;
        let receive_ring = alloc(receive_ring_length * 16) as *mut RD;
        for i in 0..receive_ring_length {
            let receive_buffer = alloc(16384);
            ptr::write(receive_ring.offset(i as isize), RD {
                buffer: receive_buffer as u64,
                length: 0,
                checksum: 0,
                status: 0,
                error: 0,
                special: 0
            });
        }

        self.write(RDBAH, 0);
        self.write(RDBAL, receive_ring as u32);
        self.write(RDLEN, (receive_ring_length * 16) as u32);
        self.write(RDH, 0);
        self.write(RDT, receive_ring_length as u32 - 1);

        self.flag(RCTL, RCTL_EN, true);
        self.flag(RCTL, RCTL_UPE, true);
        self.flag(RCTL, RCTL_LPE, true);
        self.flag(RCTL, RCTL_LBM, false);
        /* RCTL.RDMTS = Minimum threshold size ??? */
        /* RCTL.MO = Multicast offset */
        self.flag(RCTL, RCTL_BAM, true);
        self.flag(RCTL, RCTL_BSIZE1, true);
        self.flag(RCTL, RCTL_BSIZE2, false);
        self.flag(RCTL, RCTL_BSEX, true);
        self.flag(RCTL, RCTL_SECRC, true);

        //Transmit Buffer
        let transmit_ring_length = 64;
        let transmit_ring = alloc(transmit_ring_length * 16) as *mut TD;
        for i in 0..transmit_ring_length {
            let transmit_buffer = alloc(16384);
            ptr::write(transmit_ring.offset(i as isize), TD {
                buffer: transmit_buffer as u64,
                length: 0,
                cso: 0,
                command: 0,
                status: 0,
                css: 0,
                special: 0
            });
        }

        self.write(TDBAH, 0);
        self.write(TDBAL, transmit_ring as u32);
        self.write(TDLEN, (transmit_ring_length * 16) as u32);
        self.write(TDH, 0);
        self.write(TDT, 0);

        self.flag(TCTL, TCTL_EN, true);
        self.flag(TCTL, TCTL_PSP, true);
        /* TCTL.CT = Collition threshold */
        /* TCTL.COLD = Collision distance */
        /* TIPG Packet Gap */
        /* TODO ... */

        self.write(IMS, IMS_RXT | IMS_RX | IMS_RXDMT | IMS_RXSEQ | IMS_LSC | IMS_TXQE | IMS_TXDW);
    }
}
