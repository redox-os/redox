use core::ops::DerefMut;

use common::memory::*;
use common::pci::*;
use common::pio::*;
use common::scheduler::*;

use network::common::*;
use network::ethernet::*;

use programs::common::*;

pub struct RTL8139Resource {
    pub nic: *mut RTL8139,
    pub ptr: *mut RTL8139Resource,
    pub inbound: Queue<Vec<u8>>,
    pub outbound: Queue<Vec<u8>>
}

impl RTL8139Resource {
    pub fn new(nic: &mut RTL8139) -> Box<RTL8139Resource> {
        let mut ret = box RTL8139Resource {
            nic: nic,
            ptr: 0 as *mut RTL8139Resource,
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

impl Resource for RTL8139Resource {
    fn url(&self) -> URL {
        return URL::from_string(&"network://".to_string());
    }

    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        d("TODO: Implement read for RTL8139\n");
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

impl Drop for RTL8139Resource {
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

pub struct TXD {
    pub address_port: u16,
    pub status_port: u16,
    pub first: bool,
    pub buffer: usize
}

pub struct RTL8139 {
    pub bus: usize,
    pub slot: usize,
    pub func: usize,
    pub base: usize,
    pub memory_mapped: bool,
    pub irq: u8,
    pub resources: Vec<*mut RTL8139Resource>,
    pub txds: Vec<TXD>,
    pub txd_i: usize
}

impl SessionItem for RTL8139 {
    fn scheme(&self) -> String {
        return "network".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        return RTL8139Resource::new(self);
    }

    fn on_irq(&mut self, irq: u8){
        if irq == self.irq {
            unsafe {
                let base = self.base as u16;

                let icr = inw(base + 0x3E);
                outw(base + 0x3E, icr);
                
                dh(icr as usize);
                dl();

                self.receive_inbound();
                self.send_outbound();
            }
        }
    }
}

impl RTL8139 {
    pub unsafe fn receive_inbound(&mut self) {
        let reenable = start_no_ints();

        let base = self.base as u16;

        let receive_buffer = ind(base + 0x30) as usize;
        let mut capr = (inw(base + 0x38) + 16) as usize;
        let cbr = inw(base + 0x3A) as usize;

        while capr != cbr {
            let frame_addr = receive_buffer + capr + 4;
            let frame_status = ptr::read((receive_buffer + capr) as *const u16) as usize;
            let frame_len = ptr::read((receive_buffer + capr + 2) as *const u16) as usize;

            d("Recv ");
            dh(capr as usize);
            d(" ");
            dh(frame_status);
            d(" ");
            dh(frame_addr);
            d(" ");
            dh(frame_len);
            dl();

            for resource in self.resources.iter() {
                (**resource).inbound.push(Vec::from_raw_buf(frame_addr as *const u8, frame_len - 4));
            }

            capr = capr + frame_len + 4;
            capr = (capr + 3) & (0xFFFFFFFF - 3);
            if capr >= 8192 {
                capr -= 8192
            }

            outw(base + 0x38, (capr as u16) - 16);
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
            let base = self.base as u16;

            loop {
                if let Option::Some(txd) = self.txds.get(self.txd_i) {
                    let tx_status = ind(txd.status_port);
                    if tx_status & (1 << 15 | 1 << 13) == (1 << 15 | 1 << 13) || (txd.first && (tx_status & 1 << 13) == (1 << 13)) {
                        let mut found = false;

                        for resource in self.resources.iter() {
                            if ! found {
                                match (**resource).outbound.pop() {
                                    Option::Some(bytes) => {
                                        if bytes.len() < 4096 {
                                            found = true;

                                            d("Send ");
                                            dh(txd.status_port as usize);
                                            d(" ");
                                            dh(tx_status as usize);
                                            d(" ");
                                            dh(txd.buffer);
                                            d(" ");
                                            dh(bytes.len() & 0xFFF);
                                            dl();

                                            ::memcpy(txd.buffer as *mut u8, bytes.as_ptr(), bytes.len());

                                            outd(txd.address_port, txd.buffer as u32);
                                            outd(txd.status_port, bytes.len() as u32 & 0xFFF);

                                            txd.first = false;
                                        }else{
                                            dl();
                                            d("RTL8139: Frame too long for transmit: ");
                                            dd(bytes.len());
                                            dl();
                                        }
                                    },
                                    Option::None => continue
                                }
                            }
                        }

                        if found {
                            self.txd_i = (self.txd_i + 1) % 4;
                        }else{
                            break;
                        }
                    }else{
                        break;
                    }
                }else{
                    d("RTL8139: TXD Overflow!\n");
                    self.txd_i = 0;
                }
            }
        }

        end_no_ints(reenable);
    }

    pub unsafe fn init(&mut self){
        d("RTL8139 on: ");
        dh(self.base);
        if self.memory_mapped {
            d(" memory mapped");
        }else{
            d(" port mapped");
        }
        d(" IRQ: ");
        dbh(self.irq);

        pci_write(self.bus, self.slot, self.func, 0x04, pci_read(self.bus, self.slot, self.func, 0x04) | (1 << 2)); // Bus mastering

        let base = self.base as u16;

        outb(base + 0x52, 0);

        outb(base + 0x37, 0x10);
        while inb(base + 0x37) & 0x10 != 0 {}

        let receive_buffer = alloc(10240);
        outd(base + 0x30, receive_buffer as u32);
        d(" RBSTART: ");
        dh(ind(base + 0x30) as usize);

        for i in 0..4 {
            self.txds.push(TXD {
                address_port: base + 0x20 + (i as u16) * 4,
                status_port: base + 0x10 + (i as u16) * 4,
                first: true,
                buffer: alloc(4096)
            });
        }

        outw(base + 0x3C, 5);
        d(" IMR: ");
        dh(inw(base + 0x3C) as usize);

        outb(base + 0x37, 0xC);
        d(" CMD: ");
        dbh(inb(base + 0x37));

        outd(base + 0x44, 0x80 | (1 << 4) | (1 << 3) | (1 << 1));
        d(" RCR: ");
        dh(ind(base + 0x44) as usize);

        d(" MAC: ");
        let mac_low = ind(base);
        let mac_high = ind(base + 4);
        let mac = MACAddr{
            bytes: [
                mac_low as u8,
                (mac_low >> 8) as u8,
                (mac_low >> 16) as u8,
                (mac_low >> 24) as u8,
                mac_high as u8,
                (mac_high >> 8) as u8
            ]
        };
        mac.d();

        dl();
    }
}
