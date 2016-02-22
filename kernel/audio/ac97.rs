use alloc::boxed::Box;

use arch::context::context_switch;
use arch::memory;

use core::{cmp, ptr, mem};

use common::time;

use drivers::pci::config::PciConfig;
use drivers::io::{Io, Pio};

use fs::{KScheme, Resource, Url};

use syscall::{do_sys_nanosleep, Result, TimeSpec};

#[repr(packed)]
struct BD {
    ptr: u32,
    samples: u32,
}

struct Ac97Resource {
    audio: usize,
    bus_master: usize,
}

impl Resource for Ac97Resource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box Ac97Resource {
            audio: self.audio,
            bus_master: self.bus_master,
        })
    }

    fn path(&self, buf: &mut [u8]) -> Result <usize> {
        let path = b"audio:";

        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }

        Ok(i)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        unsafe {
            let audio = self.audio as u16;

            let mut master_volume = Pio::<u16>::new(audio + 2);
            let mut pcm_volume = Pio::<u16>::new(audio + 0x18);

            master_volume.write(0x808);
            pcm_volume.write(0x808);

            let bus_master = self.bus_master as u16;

            let mut po_bdbar = Pio::<u32>::new(bus_master + 0x10);
            let po_civ = Pio::<u8>::new(bus_master + 0x14);
            let mut po_lvi = Pio::<u8>::new(bus_master + 0x15);
            let mut po_cr = Pio::<u8>::new(bus_master + 0x1B);

            loop {
                if po_cr.read() & 1 == 0 {
                    break;
                }
                context_switch();
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

                    let req = TimeSpec {
                        tv_sec: 0,
                        tv_nsec: 10 * time::NANOS_PER_MILLI
                    };
                    let mut rem = TimeSpec {
                        tv_sec: 0,
                        tv_nsec: 0,
                    };
                    try!(do_sys_nanosleep(&req, &mut rem));
                }

                debug!("{} / {}: {} / {}\n",
                       po_civ.read(),
                       lvi as usize,
                       position,
                       buf.len());

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

                let req = TimeSpec {
                    tv_sec: 0,
                    tv_nsec: 10 * time::NANOS_PER_MILLI
                };
                let mut rem = TimeSpec {
                    tv_sec: 0,
                    tv_nsec: 0,
                };
                try!(do_sys_nanosleep(&req, &mut rem));
            }

            debug!("Finished {} / {}\n", po_civ.read(), lvi);
        }

        Ok(buf.len())
    }
}

pub struct Ac97 {
    pub audio: usize,
    pub bus_master: usize,
    pub irq: u8,
}

impl KScheme for Ac97 {
    fn scheme(&self) -> &str {
        "audio"
    }

    fn open(&mut self, _: &Url, _: usize) -> Result<Box<Resource>> {
        Ok(box Ac97Resource {
            audio: self.audio,
            bus_master: self.bus_master,
        })
    }

    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            // d("AC97 IRQ\n");
        }
    }

    fn on_poll(&mut self) {}
}

impl Ac97 {
    pub unsafe fn new(mut pci: PciConfig) -> Box<Ac97> {
        pci.flag(4, 4, true); // Bus mastering

        let module = box Ac97 {
            audio: pci.read(0x10) as usize & 0xFFFFFFF0,
            bus_master: pci.read(0x14) as usize & 0xFFFFFFF0,
            irq: pci.read(0x3C) as u8 & 0xF,
        };

        debug!("AC97 on: {:X}, {:X}, IRQ: {:X}\n",
               module.audio,
               module.bus_master,
               module.irq);

        module
    }
}
