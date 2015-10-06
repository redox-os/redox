use alloc::boxed::Box;

use core::{cmp, ptr, mem};

use common::debug;
use common::memory;
use common::resource::{Resource, ResourceSeek, ResourceType, URL};
use common::string::{String, ToString};
use common::time::{self, Duration};

use drivers::pciconfig::*;
use drivers::pio::*;

use programs::common::SessionItem;

use syscall::call;

#[repr(packed)]
struct BD {
    ptr: u32,
    samples: u32,
}

struct AC97Resource {
    audio: usize,
    bus_master: usize,
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

            let mut master_volume = PIO16::new(audio + 2);
            let mut pcm_volume = PIO16::new(audio + 0x18);

            master_volume.write(0x808);
            pcm_volume.write(0x808);

            let bus_master = self.bus_master as u16;

            let mut po_bdbar = PIO32::new(bus_master + 0x10);
            let po_civ = PIO8::new(bus_master + 0x14);
            let mut po_lvi = PIO8::new(bus_master + 0x15);
            let po_sr = PIO16::new(bus_master + 0x16);
            let po_picb = PIO16::new(bus_master + 0x18);
            let po_piv = PIO8::new(bus_master + 0x1A);
            let mut po_cr = PIO8::new(bus_master + 0x1B);

            loop {
                if po_cr.read() & 1 == 0 {
                    break;
                }
                Duration::new(0, 10 * time::NANOS_PER_MILLI).sleep();
            }

            po_cr.write(0);

            let mut bdl = po_bdbar.read() as *mut BD;
            if bdl as usize == 0 {
                bdl = memory::alloc(32 * mem::size_of::<BD>()) as *mut BD;
                po_bdbar.write(bdl as u32);
            }

            for i in 0..32 {
                ptr::write(bdl.offset(i),
                           BD {
                               ptr: 0,
                               samples: 0,
                           });
            }

            let mut wait = false;
            let mut position = 0;

            let mut lvi = po_lvi.read();

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
                    if po_civ.read() != lvi as u8 {
                        break;
                    }
                    Duration::new(0, 10 * time::NANOS_PER_MILLI).sleep();
                }

                debug::dd(po_civ.read() as usize);
                debug::d(" / ");
                debug::dd(lvi as usize);
                debug::d(": ");
                debug::dd(position);
                debug::d(" / ");
                debug::dd(buf.len());
                debug::dl();

                let bytes = cmp::min(65534 * 2, (buf.len() - position + 1));
                let samples = bytes / 2;

                ptr::write(bdl.offset(lvi as isize),
                           BD {
                               ptr: buf.as_ptr().offset(position as isize) as u32,
                               samples: (samples & 0xFFFF) as u32,
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
                    po_lvi.write(start_lvi);
                    po_cr.write(1);
                    wait = true;
                }
            }

            po_lvi.write(lvi);
            po_cr.write(1);

            loop {
                if po_civ.read() == lvi {
                    po_cr.write(0);
                    break;
                }
                Duration::new(0, 10 * time::NANOS_PER_MILLI).sleep();
            }

            debug::d("Finished ");
            debug::dd(po_civ.read() as usize);
            debug::d(" / ");
            debug::dd(lvi as usize);
            debug::dl();
        }

        Option::Some(buf.len())
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        Option::None
    }

    fn sync(&mut self) -> bool {
        false
    }
}

pub struct AC97 {
    pub audio: usize,
    pub bus_master: usize,
    pub irq: u8,
}

impl SessionItem for AC97 {
    fn scheme(&self) -> String {
        "audio".to_string()
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        box AC97Resource {
            audio: self.audio,
            bus_master: self.bus_master,
        }
    }

    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            //d("AC97 IRQ\n");
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
            irq: pci.read(0x3C) as u8 & 0xF,
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
