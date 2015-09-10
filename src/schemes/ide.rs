use alloc::arc::*;

use common::memory::*;
use common::pci::*;
use common::pio::*;
use common::scheduler::*;

use drivers::disk::*;

use programs::common::*;

pub struct IDERequest {
    pub sector: u64,
    pub count: u64,
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

impl SessionItem for IDE {
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
                d("IDE");

                d(" Status ");
                dbh(status);

                outb(base + 0x2, status);

                d(" to ");
                dbh(inb(base + 0x2));

                if command & 1 == 1 && status & 1 == 0 {
                    d(" DMA Command ");
                    dbh(command);

                    outb(base, 0);

                    d(" to ");
                    dbh(inb(base));

                    let prdt = ind(base + 0x4) as usize;
                    outd(base + 0x4, 0);

                    d(" PRDT ");
                    dh(prdt);

                    if prdt > 0 {
                        let destination = (*(prdt as *const PRDTE)).ptr as usize;
                        unalloc(prdt);

                        match self.requests.remove(0){
                            Option::Some(request) => {
                                (request.callback)(destination);
                            },
                            Option::None => ()
                        }

                        match self.requests.get(0) {
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
                    }
                }

                dl();
            }
        }
    }

    fn scheme(&self) -> String {
        return "ide".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        let data: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0xFFFFFFFF));
        let data_callback = data.clone();

        let mut request = IDERequest {
            sector: 1,
            count: 1,
            destination: 0,
            callback: box move |destination: usize|{
                data_callback.store(destination, Ordering::SeqCst);
            }
        };

        let mut i = 0;
        for part in url.path_parts().iter() {
            match i {
                0 => request.sector = part.to_num() as u64,
                1 => request.count = part.to_num() as u64,
                _ => ()
            }
            i += 1;
        }

        let url = URL::from_string(&("ide:///".to_string() + request.sector as usize + "/" + request.count as usize));

        unsafe {
            if request.count > 0 {
                request.destination = alloc(request.count as usize * 512);
                if request.destination > 0 {
                    let reenable = start_no_ints();
                    self.requests.push(request);
                    if self.requests.len() == 1 {
                        match self.requests.get(0) {
                            Option::Some(request) => {
                                d("IDE DMA Request ");
                                dd(request.sector as usize);
                                d(" ");
                                dd(request.count as usize);

                                let disk = Disk::primary_master();
                                disk.read_dma(request.sector, request.count, request.destination, self.base as u16);

                                dl();
                            },
                            Option::None => ()
                        }
                    }
                    end_no_ints(reenable);
                }
            }

            while data.load(Ordering::SeqCst) == 0xFFFFFFFF {
                sys_yield();
            }

            let destination = data.load(Ordering::SeqCst);
            if destination > 0 {
                return box VecResource::new(url, ResourceType::File, Vec::<u8> {
                    data: destination as *mut u8,
                    length: alloc_size(destination)
                });
            }else{
                return box NoneResource;
            }
        }
    }
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

        d(" PRDT ");
        dh(ind(base + 0x4) as usize);

        d(" CMD ");
        dbh(inb(base));

        d(" STS ");
        dbh(inb(base + 0x2));

        dl();
    }
}
