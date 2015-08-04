use common::debug::*;
use common::memory::*;
use common::pci::*;
use common::pio::*;
use common::string::*;
use common::url::*;

use drivers::disk::*;

use programs::session::*;


pub struct IDEScheme {
    pub bus: usize,
    pub slot: usize,
    pub func: usize,
    pub base: usize,
    pub memory_mapped: bool
}

impl SessionScheme for IDEScheme {
    fn scheme(&self) -> String {
        return "ide".to_string();
    }

    #[allow(unused_variables)]
    fn on_url(&mut self, session: &Session, url: &URL) -> String {
        unsafe {
            let destination = alloc(512);
            let disk = Disk::new();
            disk.read_dma(1, 1, destination, self.base as u16);
        }

        return "TEST".to_string();
    }
}

pub struct IDE {
    pub bus: usize,
    pub slot: usize,
    pub func: usize,
    pub base: usize,
    pub memory_mapped: bool
}

impl SessionDevice for IDE {
    #[allow(unused_variables)]
    fn on_irq(&mut self, session: &Session, updates: &mut SessionUpdates, irq: u8){
        if irq == 0xE || irq == 0xF {
            unsafe {
                let base = self.base as u16;

                let command = inb(base);
                let status = inb(base + 2);
                if status & 4 == 4 {
                    d("IDE handle");

                    if command & 1 == 1 {
                        d(" Command ");
                        dbh(command);

                        outb(base, command & 0xFE);

                        d(" to ");
                        dbh(inb(base));

                        d(" Status ");
                        dbh(status);

                        outb(base + 0x2, 1);

                        d(" to ");
                        dbh(inb(base + 0x2));

                        let prdt = ind(base + 0x4) as usize;
                        if prdt > 0 {
                            let prdte = &mut *(prdt as *mut PRDTE);
                            d(" PTR ");
                            dh(prdte.ptr as usize);

                            d(" SIZE ");
                            dd(prdte.size as usize);

                            d(" RESERVED ");
                            dh(prdte.reserved as usize);

                            d(" DATA ");
                            for i in 0..4 {
                                dc(*(prdte.ptr as *const u8).offset(i) as char);
                            }

                            unalloc(prdte.ptr as usize);
                            prdte.ptr = 0;
                        }
                        unalloc(prdt);

                        outd(base + 0x4, 0);
                    }

                    dl();
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
