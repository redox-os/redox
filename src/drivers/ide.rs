use core::result::Result;

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
            let base = self.base as u16;

            if count < 1 {
                count = 1;
            }

            if count > 0x7F {
                count = 0x7F;
            }

            let destination = alloc(count * 512);
            if destination > 0 {
                let disk = Disk::new();
                //disk.read(sector as u64, count as u16, destination);
                disk.read_dma(sector as u64, count as u16, destination, base);

                while inb(base + 2) & 4 != 4{
                    d("DISK WAIT\n");
                }

                ret = String::from_c_str(destination as *const u8);
                unalloc(destination);
            }
        }

        return ret;
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

                        unalloc(prdt);
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
