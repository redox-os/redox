use core::option::Option;

use alloc::boxed::*;

use common::debug::*;
use common::memory::*;
use common::pci::*;
use common::pio::*;
use common::resource::*;
use common::string::*;
use common::vec::*;
use common::url::*;

use drivers::disk::*;

use programs::session::*;

pub struct IDERequest {
    pub sector: u64,
    pub count: u16,
    pub destination: usize,
    pub callback: Box<FnBox(usize)>
}

pub struct IDE {
    pub bus: usize,
    pub slot: usize,
    pub func: usize,
    pub base: usize,
    pub memory_mapped: bool,
    pub requests: Vec<IDERequest>
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
                            Option::Some(request) => {
                                d(" NEXT ");
                                dd(request.sector as usize);
                                d(" ");
                                dd(request.count as usize);

                                let disk = Disk::new();
                                disk.read_dma(request.sector, request.count, request.destination, base);
                            },
                            Option::None => ()
                        }

                        match self.requests.remove(0){
                            Option::Some(request) => {
                                d(" REQUEST ");
                                dh(request.destination);
                                d(" RETURN ");
                                dh(destination);
                                (request.callback)(destination);
                            },
                            Option::None => ()
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

    fn open(&mut self, url: &URL) -> Box<Resource> {
        let mut sector = 1;
        match url.path.get(0) {
            Option::Some(part) => {
                sector = part.to_num() as u64;
            },
            Option::None => ()
        }

        let mut count = 1;
        match url.path.get(1) {
            Option::Some(part) => {
                count = part.to_num() as u16;
            },
            Option::None => ()
        }

        unsafe {
            if count > 0 {
                let destination = alloc(count as usize * 512);
                if destination > 0 {
                    let disk = Disk::new();
                    disk.read(sector, count, destination);
                    return box VecResource::new(Vec::<u8> {
                        data: destination as *mut u8,
                        length: alloc_size(destination)
                    });
                }
            }
        }

        return box NoneResource;
    }

    #[allow(unused_variables)]
    fn open_async(&mut self, url: &URL, callback: Box<FnBox(Box<Resource>)>){
        let mut request = IDERequest {
            sector: 1,
            count: 1,
            destination: 0,
            callback: box move |destination: usize|{
                if destination > 0 {
                    unsafe{
                        callback(box VecResource::new(Vec::<u8> {
                            data: destination as *mut u8,
                            length: alloc_size(destination)
                        }));
                    }
                }else{
                    callback(box NoneResource);
                }
            }
        };

        match url.path.get(0) {
            Option::Some(part) => {
                request.sector = part.to_num() as u64;
            },
            Option::None => ()
        }

        match url.path.get(1) {
            Option::Some(part) => {
                request.count = part.to_num() as u16;
            },
            Option::None => ()
        }

        unsafe {
            if request.count > 0 {
                request.destination = alloc(request.count as usize * 512);
                if request.destination > 0 {
                    self.requests.push(request);
                    if self.requests.len() == 1 {
                        match self.requests.get(0) {
                            Option::Some(request) => {
                                let disk = Disk::new();
                                disk.read_dma(request.sector, request.count, request.destination, self.base as u16);
                            },
                            Option::None => ()
                        }
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
