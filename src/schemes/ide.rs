use core::cmp::max;
use core::cmp::min;

use common::memory::*;
use common::pci::*;
use common::pio::*;

use drivers::disk::*;

use programs::common::*;

struct IDEResource {
    disk: Disk,
    sector: u64,
    count: u16,
    vec: Vec<u8>,
    seek: usize,
    changed: bool
}

impl IDEResource {
    fn new(disk: Disk, sector: u64, count: u16) -> IDEResource {
        let vec;
        unsafe{
            let destination = alloc(count as usize * 512);
            if destination > 0 {
                disk.read(sector, count, destination);
                vec = Vec {
                    data: destination as *mut u8,
                    length: count as usize * 512
                };
            }else{
                vec = Vec::new();
            }
        }

        return IDEResource {
            disk: disk,
            sector: sector,
            count: count,
            vec: vec,
            seek: 0,
            changed: false
        };
    }
}

impl Resource for IDEResource {
    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.vec.len() {
            match self.vec.get(self.seek) {
                Option::Some(b) => buf[i] = *b,
                Option::None => ()
            }
            self.seek += 1;
            i += 1;
        }
        return Option::Some(i);
    }

    fn read_to_end(&mut self, vec: &mut Vec<u8>) -> Option<usize> {
        vec.push_all(&self.vec);
        return Option::Some(self.vec.len());
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        let mut i = 0;
        while i < buf.len() && self.seek < self.vec.len() {
            self.vec.set(self.seek, buf[i]);
            self.seek += 1;
            i += 1;
        }
        if i > 0 {
            self.changed = true;
        }
        return Option::Some(i);
    }

    fn write_all(&mut self, vec: &Vec<u8>) -> Option<usize> {
        let mut i = 0;
        while i < vec.len() && self.seek < self.vec.len() {
            match vec.get(i) {
                Option::Some(b) => {
                    self.vec.set(self.seek, *b);
                    self.seek += 1;
                    i += 1;
                },
                Option::None => break
            }
        }
        if i > 0 {
            self.changed = true;
        }
        return Option::Some(i);
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        match pos {
            ResourceSeek::Start(offset) => self.seek = min(self.seek, offset),
            ResourceSeek::End(offset) => self.seek = max(0, min(self.seek as isize, self.vec.len() as isize + offset)) as usize,
            ResourceSeek::Current(offset) => self.seek = max(0, min(self.seek as isize, self.seek as isize + offset)) as usize
        }
        return Option::Some(self.seek);
    }

    fn flush(&mut self) -> bool {
        if self.changed {
            unsafe{
                d("Flush IDE\n");
                self.disk.write(self.sector, self.count, self.vec.data as usize);
            }
            self.changed = false;
            return true;
        }else{
            return false;
        }
    }
}

impl Drop for IDEResource {
    fn drop(&mut self){
        self.flush();
    }
}

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
    fn on_irq(&mut self, irq: u8){
        if irq == 0xE || irq == 0xF {
            self.on_poll();
        }
    }

    fn on_poll(&mut self){
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

                                let disk = Disk::primary_master();
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
        let mut count = 1;

        let mut i = 0;
        for part in url.path.split("/".to_string()) {
            match i {
                0 => sector = part.to_num() as u64,
                1 => count = part.to_num() as u16,
                _ => ()
            }
            i += 1;
        }

        unsafe {
            if count > 0 {
                return box IDEResource::new(Disk::primary_master(), sector, count);
            }
        }

        return box NoneResource;
    }
/*
    #[allow(unused_variables)]
    fn open_async(&mut self, url: &URL, callback: Box<FnBox(Box<Resource>)>){
        let mut request = IDERequest {
            sector: 1,
            count: 1,
            destination: 0,
            callback: box move |destination: usize|{
                if destination > 0 {
                    unsafe{
                        callback(box VecResource::new(ResourceType::File, Vec::<u8> {
                            data: destination as *mut u8,
                            length: alloc_size(destination)
                        }));
                    }
                }else{
                    callback(box NoneResource);
                }
            }
        };

        let mut i = 0;
        for part in url.path.split("/".to_string()) {
            match i {
                0 => request.sector = part.to_num() as u64,
                1 => request.count = part.to_num() as u16,
                _ => ()
            }
            i += 1;
        }

        unsafe {
            if request.count > 0 {
                request.destination = alloc(request.count as usize * 512);
                if request.destination > 0 {
                    self.requests.push(request);
                    if self.requests.len() == 1 {
                        match self.requests.get(0) {
                            Option::Some(request) => {
                                let disk = Disk::primary_master();
                                disk.read_dma(request.sector, request.count, request.destination, self.base as u16);
                            },
                            Option::None => ()
                        }
                    }
                }
            }
        }
    }
*/
}

impl IDE {
    pub unsafe fn init(&self){
        d("IDE on: ");
        dh(self.base);
        if self.memory_mapped {
            d(" memory mapped");
        }else{
            d(" port mapped");
        }

        pci_write(self.bus, self.slot, self.func, 0x04, pci_read(self.bus, self.slot, self.func, 0x04) | (1 << 2)); // Bus mastering

        let base = self.base as u16;

        d(" PDTR ");
        dh(ind(base + 0x4) as usize);

        d(" CMD ");
        dbh(inb(base));

        d(" STS ");
        dbh(inb(base + 0x2));

        dl();
    }
}
