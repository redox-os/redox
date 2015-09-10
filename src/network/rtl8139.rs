use common::memory::*;
use common::pci::*;
use common::pio::*;
use common::scheduler::*;

use network::common::*;
use network::scheme::*;

use programs::common::*;

pub struct TXD {
    pub address_port: u16,
    pub status_port: u16,
    pub buffer: usize
}

pub struct RTL8139 {
    pub bus: usize,
    pub slot: usize,
    pub func: usize,
    pub base: usize,
    pub memory_mapped: bool,
    pub irq: u8,
    pub resources: Vec<*mut NetworkResource>,
    pub inbound: Queue<Vec<u8>>,
    pub outbound: Queue<Vec<u8>>,
    pub txds: Vec<TXD>,
    pub txd_i: usize
}

impl SessionItem for RTL8139 {
    fn scheme(&self) -> String {
        return "network".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        return NetworkResource::new(self);
    }

    fn on_irq(&mut self, irq: u8){
        if irq == self.irq {
            unsafe {
                let base = self.base as u16;

                let isr = inw(base + 0x3E);
                outw(base + 0x3E, isr);

                dh(isr as usize);
                dl();
            }

            self.sync();
        }
    }

    fn on_poll(&mut self){
        self.sync();
    }
}

impl NetworkScheme for RTL8139 {
    fn add(&mut self, resource: *mut NetworkResource){
        unsafe {
            let reenable = start_no_ints();
            self.resources.push(resource);
            end_no_ints(reenable);
        }
    }

    fn remove(&mut self, resource: *mut NetworkResource){
        unsafe {
            let reenable = start_no_ints();
            let mut i = 0;
            while i < self.resources.len() {
                let mut remove = false;

                match self.resources.get(i) {
                    Option::Some(ptr) => if *ptr == resource {
                        remove = true;
                    }else{
                        i += 1;
                    },
                    Option::None => break
                }

                if remove {
                    self.resources.remove(i);
                }
            }
            end_no_ints(reenable);
        }
    }

    fn sync(&mut self){
        unsafe {
            let reenable = start_no_ints();

            for resource in self.resources.iter() {
                while let Option::Some(bytes) = (**resource).outbound.pop() {
                    self.outbound.push(bytes);
                }
            }

            self.send_outbound();

            self.receive_inbound();

            while let Option::Some(bytes) = self.inbound.pop() {
                for resource in self.resources.iter() {
                    (**resource).inbound.push(bytes.clone());
                }
            }

            end_no_ints(reenable);
        }
    }
}

impl RTL8139 {
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

        d(" MAC: ");
        let mac_low = ind(base);
        let mac_high = ind(base + 4);
        MAC_ADDR = MACAddr{
            bytes: [
                mac_low as u8,
                (mac_low >> 8) as u8,
                (mac_low >> 16) as u8,
                (mac_low >> 24) as u8,
                mac_high as u8,
                (mac_high >> 8) as u8
            ]
        };
        MAC_ADDR.d();

        let receive_buffer = alloc(10240);
        outd(base + 0x30, receive_buffer as u32);

        for i in 0..4 {
            self.txds.push(TXD {
                address_port: base + 0x20 + (i as u16) * 4,
                status_port: base + 0x10 + (i as u16) * 4,
                buffer: alloc(4096)
            });
        }

        outw(base + 0x3C, 5);
        d(" IMR: ");
        dh(inw(base + 0x3C) as usize);

        outb(base + 0x37, 0xC);
        d(" CMD: ");
        dbh(inb(base + 0x37));

        outd(base + 0x44, (1 << 7) | (1 << 4) | (1 << 3) | (1 << 2) | (1 << 1));
        d(" RCR: ");
        dh(ind(base + 0x44) as usize);

        outd(base + 0x40, (0b11 << 24));
        d(" TCR: ");
        dh(ind(base + 0x40) as usize);

        dl();
    }

    pub fn receive_inbound(&mut self) {
        unsafe {
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

                self.inbound.push(Vec::from_raw_buf(frame_addr as *const u8, frame_len - 4));

                capr = capr + frame_len + 4;
                capr = (capr + 3) & (0xFFFFFFFF - 3);
                if capr >= 8192 {
                    capr -= 8192
                }

                outw(base + 0x38, (capr as u16) - 16);
            }
        }
    }

    pub fn send_outbound(&mut self) {
        unsafe {
            while let Option::Some(bytes) = self.outbound.pop() {
                if let Option::Some(txd) = self.txds.get(self.txd_i) {
                    if bytes.len() < 4096 {
                        let mut tx_status = 0;
                        loop {
                            tx_status = ind(txd.status_port);
                            if tx_status & (1 << 13) == (1 << 13) {
                                break;
                            }
                        }

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

                        self.txd_i = (self.txd_i + 1) % 4;
                    }else{
                        dl();
                        d("RTL8139: Frame too long for transmit: ");
                        dd(bytes.len());
                        dl();
                    }
                }else{
                    d("RTL8139: TXD Overflow!\n");
                    self.txd_i = 0;
                }
            }
        }
    }
}
