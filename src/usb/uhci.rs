use core::ptr::{read, write};

use common::memory::*;
use common::pci::*;
use common::pio::*;
use common::scheduler::*;

use programs::common::*;

pub struct UHCI {
    pub base: usize,
    pub irq: u8
}

impl SessionItem for UHCI {
    fn on_irq(&mut self, irq: u8){
        if irq == self.irq {
            d("UHCI IRQ\n");
        }
    }

    fn on_poll(&mut self){
    }
}

#[repr(packed)]
struct SETUP {
    request_type: u8,
    request: u8,
    value: u16,
    index: u16,
    len: u16
}

#[repr(packed)]
struct TD {
    link_ptr: u32,
    ctrl_sts: u32,
    token: u32,
    buffer: u32
    //reserved: [u32; 4]
}

#[repr(packed)]
struct QH {
    head_ptr: u32,
    element_ptr: u32
}

impl UHCI {
    pub unsafe fn new(bus: usize, slot: usize, func: usize) -> Box<UHCI> {
        pci_write(bus, slot, func, 0x04, pci_read(bus, slot, func, 4) | 4); // Bus mastering

        let mut module = box UHCI {
            base: pci_read(bus, slot, func, 0x20) & 0xFFFFFFF0,
            irq: pci_read(bus, slot, func, 0x3C) as u8 & 0xF
        };

        module.init();

        return module;
    }

    pub unsafe fn init(&self){
        d("UHCI on: ");
        dh(self.base);
        d(", IRQ: ");
        dbh(self.irq);

        let base = self.base as u16;
        let usbcmd = base;
        let usbsts = base + 02;
        let usbintr = base + 0x4;
        let frnum = base + 0x6;
        let flbaseadd = base + 0x8;
        let sofmod = base + 0xC;
        let portsc1 = base + 0x10;
        let portsc2 = base + 0x12;

        d(" CMD ");
        dh(inw(usbcmd) as usize);

        d(" STS ");
        dh(inw(usbsts) as usize);

        d(" INTR ");
        dh(inw(usbintr) as usize);

        d(" FRNUM ");
        dh(inw(frnum) as usize);
            outw(frnum, 0);
        d(" to ");
        dh(inw(frnum) as usize);

        d(" FLBASEADD ");
        dh(ind(flbaseadd) as usize);
            let frame_list = alloc(1024 * 4) as *mut u32;
            for i in 0..1024 {
                write(frame_list.offset(i), 1);
            }
            outd(flbaseadd, frame_list as u32);
        d(" to ");
        dh(ind(flbaseadd) as usize);

        d(" PORTSC1 ");
        dh(inw(portsc1) as usize);

        if inw(portsc1) & 1 == 1 {
            outw(portsc1, 1 << 9);
            d(" to ");
            dh(inw(portsc1) as usize);

            let disable = start_ints();
            Duration::new(1, 0).sleep();
            end_ints(disable);

            outw(portsc1, 0);
            d(" to ");
            dh(inw(portsc1) as usize);

            let disable = start_ints();
            Duration::new(1, 0).sleep();
            end_ints(disable);

            outw(portsc1, 4);
            d(" to ");
            dh(inw(portsc1) as usize);
        }

        d(" PORTSC2 ");
        dh(inw(portsc2) as usize);

        if inw(portsc2) & 1 == 1 {
            outw(portsc2, 4);

            d(" to ");
            dh(inw(portsc2) as usize);
        }

        dl();

        if inw(portsc1) & 5 == 5 {
            d("Port 1 ");
            dh(inw(portsc1) as usize);

            let in_data_len = 64;
            let in_data = alloc(in_data_len) as *mut u8;
            for i in 0..in_data_len{
                write(in_data.offset(i as isize), 0);
            }

            let in_td: *mut TD = alloc_type();
            ptr::write(in_td, TD {
                link_ptr: 1,
                ctrl_sts: 1 << 23,
                token: (in_data_len as u32 - 1) << 21 | 0x69,
                buffer: in_data as u32
            });

            let setup: *mut SETUP = alloc_type();
            write(setup, SETUP {
                request_type: 0b10000000,
                request: 6,
                value: 1 << 8,
                index: 0,
                len: in_data_len as u16
            });

            let setup_td: *mut TD = alloc_type();
            write(setup_td, TD {
                link_ptr: in_td as u32 | 4,
                ctrl_sts: 1 << 23,
                token: (size_of::<SETUP>() as u32 - 1) << 21 | 0x2D,
                buffer: setup as u32
            });

            let queue_head: *mut QH = alloc_type();
            write(queue_head, QH {
                head_ptr: 1,
                element_ptr: setup_td as u32
            });

            write(frame_list.offset(0), queue_head as u32 | 2);

            d(" CMD ");
            dh(inw(usbcmd) as usize);

            d(" STS ");
            dh(inw(usbsts) as usize);

            outd(usbcmd, 1);

            dl();

            loop {
                d("PORTSC ");
                dh(inw(portsc1) as usize);

                d(" CMD ");
                dh(inw(usbcmd) as usize);

                d(" STS ");
                dh(inw(usbsts) as usize);

                d(" QH ");
                dh((*queue_head).element_ptr as usize);

                d(" SETUP ");
                dh((*setup_td).ctrl_sts as usize);

                d(" IN ");
                let in_td_sts = (*in_td).ctrl_sts;
                dh(in_td_sts as usize);


                if in_td_sts & (1 << 23) == 0 {
                    d(" DATA");
                    for i in 0..64 {
                        d(" ");
                        dbh(*in_data.offset(i));
                    }
                    dl();
                    break;
                }

                dl();

                let disable = start_ints();
                Duration::new(1, 0).sleep();
                end_ints(disable);
            }
        }
    }
}
