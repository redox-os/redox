use core::ops::Fn;
use core::result::Result;

use alloc::boxed::*;

use common::debug::*;
use common::memory::*;
use common::pci::*;
use common::pio::*;
use common::string::*;
use common::vector::*;
use common::url::*;

use drivers::disk::*;

use programs::session::*;

pub struct IDERequest {
    pub sector: u64,
    pub count: u16,
    pub destination: usize,
    pub callback: Box<Fn(usize)>
}

pub struct IDE {
    pub bus: usize,
    pub slot: usize,
    pub func: usize,
    pub base: usize,
    pub memory_mapped: bool,
    pub requests: Vector<IDERequest>
}

impl SessionModule for IDE {
    #[allow(unused_variables)]
    fn on_irq(&mut self, session: &Session, updates: &mut SessionUpdates, irq: u8){
        if irq == 0xE || irq == 0xF {
            self.on_poll(session, updates);
        }
    }

    #[allow(unused_variables)]
    fn on_poll(&mut self, session: &Session, updates: &mut SessionUpdates){
        unsafe {
            let base = self.base as u16;

            let command = inb(base);
            let status = inb(base + 2);
            if status & 4 == 4 {
                d("IDE handle");

                if command & 1 == 1 {
                    d(" DMA Command ");
                    dbh(command);

                    outb(base, command & 0xFE);

                    d(" to ");
                    dbh(inb(base));

                    d(" Status ");
                    dbh(status);

                    outb(base + 0x2, 4);

                    d(" to ");
                    dbh(inb(base + 0x2));

                    let prdt = ind(base + 0x4) as usize;
                    outd(base + 0x4, 0);

                    d(" PRDT ");
                    dh(prdt);

                    if prdt > 0 {
                        let destination = (*(prdt as *const PRDTE)).ptr as usize;
                        unalloc(prdt);

                        match self.requests.get(1) {
                            Result::Ok(request) => {
                                d(" NEXT ");
                                dd(request.sector as usize);
                                d(" ");
                                dd(request.count as usize);

                                let disk = Disk::new();
                                disk.read_dma(request.sector, request.count, request.destination, base);
                            },
                            Result::Err(_) => ()
                        }

                        match self.requests.extract(0) {
                            Result::Ok(request) => {
                                d(" REQUEST ");
                                dh(request.destination);
                                d(" RETURN ");
                                dh(destination);
                                (*request.callback)(destination);
                            },
                            Result::Err(_) => ()
                        }
                    }
                }else{
                    d(" PIO Command ");
                    dbh(command);

                    d(" Status ");
                    dbh(status);

                    outb(base + 0x2, 4);

                    d(" to ");
                    dbh(inb(base + 0x2));
                }

                dl();
            }
        }
    }

    fn scheme(&self) -> String {
        return "ide".to_string();
    }

    #[allow(unused_variables)]
    fn on_url(&mut self, session: &Session, url: &URL) -> String{
        let mut ret = String::new();

        let mut sector = 1;
        match url.path.get(0) {
            Result::Ok(part) => {
                sector = part.to_num();
            },
            Result::Err(_) => ()
        }

        let mut count = 1;
        match url.path.get(1) {
            Result::Ok(part) => {
                count = part.to_num();
            },
            Result::Err(_) => ()
        }

        unsafe {
            if count > 0 {
                if count > 0xFFFF {
                    count = 0xFFFF;
                }

                let destination = alloc(count * 512);
                if destination > 0 {
                    let disk = Disk::new();
                    disk.read(sector as u64, count as u16, destination);

                    ret = String::from_num_radix(destination, 16);
                }
            }
        }

        return ret;
    }

    #[allow(unused_variables)]
    fn on_url_async(&mut self, session: &Session, url: &URL, callback: Box<Fn(String)>){
        let mut request = IDERequest {
            sector: 1,
            count: 1,
            destination: 0,
            callback: box move |destination: usize|{
                if destination > 0 {
                    callback(String::from_num(destination));
                }else{
                    callback(String::new());
                }
            }
        };

        match url.path.get(0) {
            Result::Ok(part) => {
                request.sector = part.to_num() as u64;
            },
            Result::Err(_) => ()
        }

        match url.path.get(1) {
            Result::Ok(part) => {
                request.count = part.to_num() as u16;
            },
            Result::Err(_) => ()
        }

        unsafe {
            d("Request ");
            dd(request.sector as usize);
            d(" ");
            dd(request.count as usize);
            dl();
            if request.count > 0 {
                request.destination = alloc(request.count as usize * 512);
                if request.destination > 0 {
                    self.requests.push(request);
                    if self.requests.len() == 1 {
                        d("Request Start\n");
                        match self.requests.get(0) {
                            Result::Ok(request) => {
                                let disk = Disk::new();
                                disk.read_dma(request.sector, request.count, request.destination, self.base as u16);
                            },
                            Result::Err(_) => ()
                        }
                    }else{
                        d("Request Wait\n");
                    }
                }
            }
        }
    }
}

impl IDE {
    pub unsafe fn init(&self){
        d("IDE Controller on ");
        dh(self.base);
        if self.memory_mapped {
            d(" memory mapped");
        }else{
            d(" port mapped");
        }
        dl();

        pci_write(self.bus, self.slot, self.func, 0x04, pci_read(self.bus, self.slot, self.func, 0x04) | (1 << 2)); // Bus mastering

        let base = self.base as u16;

        d("PDTR: ");
        dh(ind(base + 0x4) as usize);
        dl();

        d("COMMAND: ");
        dbh(inb(base));
        dl();

        d("STATUS: ");
        dbh(inb(base + 0x2));
        dl();
    }
}
