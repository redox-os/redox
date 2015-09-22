use core::intrinsics::{volatile_load, volatile_store};
use core::ptr::{read, write};

use common::memory::*;
use common::pci::*;
use common::scheduler::*;

use programs::common::*;

#[repr(packed)]
struct SETUP {
    request_type: u8,
    request: u8,
    value: u16,
    index: u16,
    len: u16
}

#[repr(packed)]
struct QTD {
    next: u32,
    next_alt: u32,
    token: u32,
    buffers: [u32; 5]
}

#[repr(packed)]
struct QueueHead {
    next: u32,
    characteristics: u32,
    capabilities: u32,
    qtd_ptr: u32,
    qtd: QTD
}

pub struct EHCI {
    pub bus: usize,
    pub slot: usize,
    pub func: usize,
    pub base: usize,
    pub memory_mapped: bool,
    pub irq: u8
}

impl SessionItem for EHCI {
    #[allow(non_snake_case)]
    fn on_irq(&mut self, irq: u8){
        if irq == self.irq {
            //d("EHCI handle");

            unsafe{
                let CAPLENGTH = self.base as *mut u8;

                let opbase = self.base + *CAPLENGTH as usize;

                let USBSTS = (opbase + 4) as *mut u32;
                //d(" USBSTS ");
                //dh(*USBSTS as usize);

                *USBSTS = 0b111111;

                //d(" USBSTS ");
                //dh(*USBSTS as usize);

                //let FRINDEX = (opbase + 0xC) as *mut u32;
                //d(" FRINDEX ");
                //dh(*FRINDEX as usize);
            }

            //dl();
        }
    }
}

impl EHCI {
    #[allow(non_snake_case)]
    pub unsafe fn init(&self){
        d("EHCI on: ");
        dh(self.base);
        if self.memory_mapped {
            d(" memory mapped");
        }else{
            d(" port mapped");
        }
        d(" IRQ: ");
        dbh(self.irq);

        pci_write(self.bus, self.slot, self.func, 0x04, pci_read(self.bus, self.slot, self.func, 0x04) | 4); // Bus master

        let CAPLENGTH = self.base as *mut u8;
        let HCSPARAMS = (self.base + 4) as *mut u32;
        let HCCPARAMS = (self.base + 8) as *mut u32;

        d(" CAPLENGTH ");
        dd(*CAPLENGTH as usize);

        d(" HCSPARAMS ");
        dh(*HCSPARAMS as usize);

        d(" HCCPARAMS ");
        dh(*HCCPARAMS as usize);

        let ports = (*HCSPARAMS & 0b1111) as usize;
        d(" PORTS ");
        dd(ports);

        let eecp = ((*HCCPARAMS >> 8) & 0xFF) as usize;
        d(" EECP ");
        dh(eecp);

        dl();

        if eecp > 0 {
            if pci_read(self.bus, self.slot, self.func, eecp) & ((1 << 24) | (1 << 16)) == (1 << 16) {
                d("Taking Ownership");
                    d(" ");
                    dh(pci_read(self.bus, self.slot, self.func, eecp));

                    pci_write(self.bus, self.slot, self.func, eecp, pci_read(self.bus, self.slot, self.func, eecp) | (1 << 24));

                    d(" ");
                    dh(pci_read(self.bus, self.slot, self.func, eecp));
                dl();

                d("Waiting");
                    d(" ");
                    dh(pci_read(self.bus, self.slot, self.func, eecp));

                    loop {
                        if pci_read(self.bus, self.slot, self.func, eecp) & ((1 << 24) | (1 << 16)) == (1 << 24) {
                            break;
                        }
                    }

                    d(" ");
                    dh(pci_read(self.bus, self.slot, self.func, eecp));
                dl();
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
            d("Halting");
                d(" CMD ");
                dh(read(USBCMD) as usize);

                d(" STS ");
                dh(read(USBSTS) as usize);

                write(USBCMD, read(USBCMD) & 0xFFFFFFF0);

                d(" CMD ");
                dh(*USBCMD as usize);

                d(" STS ");
                dh(*USBSTS as usize);
            dl();

            d("Waiting");
                loop{
                    if volatile_load(USBSTS) & (1 << 12) == (1 << 12) {
                        break;
                    }
                }

                d(" CMD ");
                dh(read(USBCMD) as usize);

                d(" STS ");
                dh(read(USBSTS) as usize);
            dl();
        }

        d("Resetting");
            d(" CMD ");
            dh(read(USBCMD) as usize);

            d(" STS ");
            dh(read(USBSTS) as usize);

            write(USBCMD, read(USBCMD) | (1 << 1));

            d(" CMD ");
            dh(read(USBCMD) as usize);

            d(" STS ");
            dh(read(USBSTS) as usize);
        dl();

        d("Waiting");
            loop{
                if volatile_load(USBCMD) & (1 << 1) == 0 {
                    break;
                }
            }

            d(" CMD ");
            dh(read(USBCMD) as usize);

            d(" STS ");
            dh(read(USBSTS) as usize);
        dl();

        d("Enabling");
            d(" CMD ");
            dh(read(USBCMD) as usize);

            d(" STS ");
            dh(read(USBSTS) as usize);

            write(USBINTR, 0b111111);

            write(USBCMD, read(USBCMD) | 1);
            write(CONFIGFLAG, 1);

            d(" CMD ");
            dh(read(USBCMD) as usize);

            d(" STS ");
            dh(read(USBSTS) as usize);
        dl();

        d("Waiting");
            loop{
                if volatile_load(USBSTS) & (1 << 12) == 0 {
                    break;
                }
            }

            d(" CMD ");
            dh(read(USBCMD) as usize);

            d(" STS ");
            dh(read(USBSTS) as usize);
        dl();

        let disable = start_ints();
        Duration::new(0, 100*NANOS_PER_MILLI).sleep();
        end_ints(disable);

        for i in 0..ports as isize {
            dd(i as usize);
            d(": ");
            dh(read(PORTSC.offset(i)) as usize);
            dl();
            
            if read(PORTSC.offset(i)) & 1 == 1 {
                d("Device on port ");
                    dd(i as usize);
                    d(" ");
                    dh(read(PORTSC.offset(i)) as usize);
                dl();

                if read(PORTSC.offset(i)) & (1 << 1) == (1 << 1) {
                    d("Connection Change");
                        d(" ");
                        dh(read(PORTSC.offset(i)) as usize);

                        write(PORTSC.offset(i), read(PORTSC.offset(i)) | (1 << 1));

                        d(" ");
                        dh(read(PORTSC.offset(i)) as usize);
                    dl();
                }

                if read(PORTSC.offset(i)) & (1 << 2) == 0 {
                    d("Reset");
                        d(" ");
                        dh(read(PORTSC.offset(i)) as usize);

                        write(PORTSC.offset(i), read(PORTSC.offset(i)) | (1 << 8));

                        d(" ");
                        dh(read(PORTSC.offset(i)) as usize);

                        write(PORTSC.offset(i), read(PORTSC.offset(i)) & 0xFFFFFEFF);

                        d(" ");
                        dh(read(PORTSC.offset(i)) as usize);
                    dl();

                    d("Wait");
                        d(" ");
                        dh(read(PORTSC.offset(i)) as usize);

                        loop{
                            if volatile_load(PORTSC.offset(i)) & (1 << 8) == 0 {
                                break;
                            }else{
                                volatile_store(PORTSC.offset(i), volatile_load(PORTSC.offset(i)) & 0xFFFFFEFF);
                            }
                        }

                        d(" ");
                        dh(read(PORTSC.offset(i)) as usize);
                    dl();
                }

                if read(PORTSC.offset(i)) & (1 << 2) == (1 << 2) {
                    d("Port Enabled ");
                    dh(read(PORTSC.offset(i)) as usize);
                    dl();

                    /*
                    let out_qtd = alloc(size_of::<QTD>()) as *mut QTD;
                    ptr::write(out_qtd, QTD {
                        next: 1,
                        next_alt: 1,
                        token: (1 << 31) | (0b11 << 10) | 0x80,
                        buffers: [0, 0, 0, 0, 0]
                    });

                    let in_data = alloc(64) as *mut u8;
                    for i in 0..64{
                        *in_data.offset(i) = 0;
                    }

                    let in_qtd = alloc(size_of::<QTD>()) as *mut QTD;
                    ptr::write(in_qtd, QTD {
                        next: out_qtd as u32,
                        next_alt: 1,
                        token: (1 << 31) | (64 << 16) | (0b11 << 10) | (0b01 << 8) | 0x80,
                        buffers: [in_data as u32, 0, 0, 0, 0]
                    });

                    let setup_packet = alloc(size_of::<SETUP>()) as *mut SETUP;
                    ptr::write(setup_packet, SETUP {
                        request_type: 0b10000000,
                        request: 6,
                        value: 1 << 8,
                        index: 0,
                        len: 64
                    });

                    let setup_qtd = alloc(size_of::<QTD>()) as *mut QTD;
                    ptr::write(setup_qtd, QTD {
                        next: in_qtd as u32,
                        next_alt: 1,
                        token: ((size_of::<SETUP>() as u32) << 16) | (0b11 << 10) | (0b10 << 8) | 0x80,
                        buffers: [setup_packet as u32, 0, 0, 0, 0]
                    });

                    let queuehead = alloc(size_of::<QueueHead>()) as *mut QueueHead;
                    ptr::write(queuehead, QueueHead {
                        next: 1,
                        characteristics: (64 << 16) | (1 << 15) | (1 << 14) | (0b10 << 12),
                        capabilities: (0b11 << 30),
                        qtd_ptr: setup_qtd as u32,
                        qtd: ptr::read(setup_qtd)
                    });

                    d("Prepare");
                        d(" CMD ");
                        dh(*USBCMD as usize);

                        d(" PTR ");
                        dh(queuehead as usize);
                    dl();

                    d("Send");
                        d(" CMD ");
                        dh(*USBCMD as usize);

                        d(" STS ");
                        dh(*USBSTS as usize);

                        *ASYNCLISTADDR = queuehead as u32;

                        d(" CMD ");
                        dh(*USBCMD as usize);

                        d(" STS ");
                        dh(*USBSTS as usize);

                        *USBCMD |= (1 << 5);

                        d(" CMD ");
                        dh(*USBCMD as usize);

                        d(" STS ");
                        dh(*USBSTS as usize);

                        *USBCMD |= 1;

                        d(" CMD ");
                        dh(*USBCMD as usize);

                        d(" STS ");
                        dh(*USBSTS as usize);
                    dl();

                    d("Wait");
                        d(" CMD ");
                        dh(*USBCMD as usize);

                        d(" STS ");
                        dh(*USBSTS as usize);
                        dl();

                        loop {
                            if *USBSTS & 0xA000  == 0 {
                                break;
                            }
                        }

                        d(" CMD ");
                        dh(*USBCMD as usize);

                        d(" STS ");
                        dh(*USBSTS as usize);
                    dl();

                    d("Stop");
                        d(" CMD ");
                        dh(*USBCMD as usize);

                        d(" STS ");
                        dh(*USBSTS as usize);

                        *USBCMD &= 0xFFFFFFFF - (1 << 5);

                        d(" CMD ");
                        dh(*USBCMD as usize);

                        d(" STS ");
                        dh(*USBSTS as usize);
                    dl();

                    d("Data");
                    for i in 0..64 {
                        d(" ");
                        dbh(*in_data.offset(i));
                    }
                    dl();

                    //Only detect one device for testing
                    break;
                    */
                }else{
                    d("Device not high-speed\n");
                }
            }
        }
    }
}
