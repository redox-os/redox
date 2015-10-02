use core::ptr::write;

use common::memory::*;

use drivers::pciconfig::*;
use drivers::pio::*;

use programs::common::*;

#[repr(packed)]
struct BD {
    ptr: u32,
    samples: u32
}

struct AC97Resource {
    audio: usize,
    bus_master: usize
}

impl Resource for AC97Resource {
    fn url(&self) -> URL {
        URL::from_str("audio://")
    }

    fn stat(&self) -> ResourceType {
        ResourceType::File
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        Option::None
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        unsafe {
            let audio = self.audio as u16;
            let master_volume = audio + 2;
            let pcm_volume = audio + 0x18;

            outw(master_volume, 0x808);
            outw(pcm_volume, 0x808);

            let bus_master = self.bus_master as u16;

            let po_bdbar = bus_master + 0x10;
            let po_civ = bus_master + 0x14;
            let po_lvi = bus_master + 0x15;
            let po_sr = bus_master + 0x16;
            let po_picb = bus_master + 0x18;
            let po_piv = bus_master + 0x1A;
            let po_cr = bus_master + 0x1B;
            let glob_cnt = bus_master + 0x2C;
            let glob_sta = bus_master + 0x30;

            loop {
                if inb(po_cr) & 1 == 0 {
                    break;
                }
                Duration::new(0, 10*NANOS_PER_MILLI).sleep();
            }

            outb(po_cr, 0);

            let mut bdl = ind(po_bdbar) as *mut BD;
            if bdl as usize == 0 {
                bdl = alloc(32 * size_of::<BD>()) as *mut BD;
                outd(po_bdbar, bdl as u32);
            }

            for i in 0..32 {
                ptr::write(bdl.offset(i), BD {
                    ptr: 0,
                    samples: 0
                });
            }

            let mut wait = false;
            let mut position = 0;


            let mut lvi = inb(po_lvi);

            let start_lvi;
            if lvi == 0 {
                start_lvi = 31;
            } else {
                start_lvi = lvi - 1;
            }

            lvi += 1;
            if lvi >= 32 {
                lvi = 0;
            }
            loop {
                while wait {
                    if inb(po_civ) != lvi as u8 {
                        break;
                    }
                    Duration::new(0, 10*NANOS_PER_MILLI).sleep();
                }

                debug::dd(inb(po_civ) as usize);
                debug::d(" / ");
                debug::dd(lvi as usize);
                debug::d(": ");
                debug::dd(position);
                debug::d(" / ");
                debug::dd(buf.len());
                debug::dl();

                let bytes = min(65534 * 2, (buf.len() - position + 1));
                let samples = bytes / 2;

                ptr::write(bdl.offset(lvi as isize), BD {
                    ptr: buf.as_ptr().offset(position as isize) as u32,
                    samples: (samples & 0xFFFF) as u32
                });

                position += bytes;

                if position >= buf.len() {
                    break;
                }

                lvi += 1;

                if lvi >= 32 {
                    lvi = 0;
                }

                if lvi == start_lvi {
                    outb(po_lvi, start_lvi);
                    outb(po_cr, 1);
                    wait = true;
                }
            }

            outb(po_lvi, lvi);
            outb(po_cr, 1);

            loop {
                if inb(po_civ) == lvi {
                    outb(po_cr, 0);
                    break;
                }
                Duration::new(0, 10*NANOS_PER_MILLI).sleep();
            }

            debug::d("Finished ");
            debug::dd(inb(po_civ) as usize);
            debug::d(" / ");
            debug::dd(lvi as usize);
            debug::dl();
        }

        Option::Some(buf.len())
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        Option::None
    }

    fn flush(&mut self) -> bool {
        false
    }
}

pub struct AC97 {
    pub audio: usize,
    pub bus_master: usize,
    pub irq: u8
}

impl SessionItem for AC97 {
    fn scheme(&self) -> String {
        "audio".to_string()
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        box AC97Resource {
            audio: self.audio,
            bus_master: self.bus_master
        }
    }

    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            //debug::d("AC97 IRQ\n");
        }
    }

    fn on_poll(&mut self) {
    }
}

impl AC97 {
    pub unsafe fn new(mut pci: PCIConfig) -> Box<AC97> {
        pci.flag(4, 4, true); // Bus mastering

        let module = box AC97 {
            audio: pci.read(0x10) as usize & 0xFFFFFFF0,
            bus_master: pci.read(0x14) as usize & 0xFFFFFFF0,
            irq: pci.read(0x3C) as u8 & 0xF
        };

        debug::d("AC97 on: ");
        debug::dh(module.audio);
        debug::d(", ");
        debug::dh(module.bus_master);
        debug::d(", IRQ: ");
        debug::dbh(module.irq);

        debug::dl();

        module
    }
}
