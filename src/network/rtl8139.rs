use common::memory::*;
use common::pci::*;
use common::pio::*;

use network::common::*;
use network::ethernet::*;

use programs::common::*;

pub struct RTL8139 {
    pub bus: usize,
    pub slot: usize,
    pub func: usize,
    pub base: usize,
    pub memory_mapped: bool,
    pub irq: u8,
    pub inbound: Queue<Vec<u8>>,
    pub outbound: Queue<Vec<u8>>
}

impl SessionItem for RTL8139 {
    fn on_irq(&mut self, irq: u8){
        if irq == self.irq {
            if cfg!(debug_network){
                d("RTL8139 handle\n");
            }

            self.on_poll();
        }
    }

    fn on_poll(&mut self){
        unsafe {
            let base = self.base as u16;

            let receive_buffer = ind(base + 0x30) as usize;
            let mut capr = (inw(base + 0x38) + 16) as usize;
            let cbr = inw(base + 0x3A) as usize;

            while capr != cbr {
                let frame_addr = receive_buffer + capr + 4;
                let frame_len = *((receive_buffer + capr + 2) as *const u16) as usize;

                if cfg!(debug_network){
                    d(" CAPR ");
                    dd(capr);
                    d(" CBR ");
                    dd(cbr);

                    d(" len ");
                    dd(frame_len);
                    dl();
                }

                self.inbound.push(Vec::from_raw_buf(frame_addr as *const u8, frame_len - 4));

                capr = capr + frame_len + 4;
                capr = (capr + 3) & (0xFFFFFFFF - 3);
                if capr >= 8192 {
                    capr -= 8192
                }

                outw(base + 0x38, (capr as u16) - 16);
            }

            //TODO: Allow preemption of this loop
            loop {
                match self.inbound.pop() {
                    Option::Some(bytes) => match EthernetII::from_bytes(bytes){
                        Option::Some(frame) => self.outbound.vec.push_all(&frame.respond()),
                        Option::None => ()
                    },
                    Option::None => break
                }
            }

            let tsd = [ind(base + 0x10), ind(base + 0x14), ind(base + 0x18), ind(base + 0x1C)];
            for i in 0..tsd.len() {
                if tsd[i] & (1 << 13) == (1 << 13) {
                    match self.outbound.pop() {
                        Option::Some(bytes) => {
                            if cfg!(debug_network){
                                d("RTL8139 send ");
                                dd(i);
                                dl();
                            }

                            outd(base + 0x20 + (i as u16)*4, bytes.as_ptr() as u32);
                            outd(base + 0x10 + (i as u16)*4, bytes.len() as u32 & 0x1FFF);
                        },
                        Option::None => break
                    }
                }
            }


            let interrupt_status = inw(base + 0x3E);

            if interrupt_status != 0 {
                outw(base + 0x3E, interrupt_status);
            }
        }
    }
}

impl RTL8139 {
    pub unsafe fn init(&self){
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

        outw(base + 0x3C, 5);
        d(" IMR: ");
        dh(inw(base + 0x3C) as usize);

        outb(base + 0x37, 0xC);
        d(" CMD: ");
        dbh(inb(base + 0x37));

        outd(base + 0x44, 0x8F);
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
