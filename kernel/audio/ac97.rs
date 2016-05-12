use alloc::boxed::Box;

use arch::context::context_switch;
use arch::memory;

use core::{cmp, mem};

use common::time;

use drivers::pci::config::PciConfig;
use drivers::io::{Io, Mmio, Pio, PhysAddr};

use fs::{KScheme, Resource, Url};

use syscall::{do_sys_nanosleep, Result, TimeSpec};

#[repr(packed)]
struct BD {
    ptr: PhysAddr<Mmio<u32>>,
    samples: Mmio<u32>,
}

struct Ac97Resource {
    audio: usize,
    bus_master: usize,
    bdl: *mut BD
}

impl Resource for Ac97Resource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box Ac97Resource {
            audio: self.audio,
            bus_master: self.bus_master,
            bdl: self.bdl
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

            for i in 0..32 {
                (*self.bdl.offset(i)).ptr.write(0);
                (*self.bdl.offset(i)).samples.write(0);
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

                (*self.bdl.offset(lvi as isize)).ptr.write(buf.as_ptr().offset(position as isize) as u32);
                (*self.bdl.offset(lvi as isize)).samples.write((samples & 0xFFFF) as u32);

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
    audio: usize,
    bus_master: usize,
    irq: u8,
    bdl: *mut BD,
}

impl KScheme for Ac97 {
    fn scheme(&self) -> &str {
        "audio"
    }

    fn open(&mut self, _: Url, _: usize) -> Result<Box<Resource>> {
        Ok(box Ac97Resource {
            audio: self.audio,
            bus_master: self.bus_master,
            bdl: self.bdl
        })
    }

    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            // d("AC97 IRQ\n");
        }
    }
}

impl Ac97 {
    pub unsafe fn new(mut pci: PciConfig) -> Box<Ac97> {
        pci.flag(4, 4, true); // Bus mastering

        let module = box Ac97 {
            audio: pci.read(0x10) as usize & 0xFFFFFFF0,
            bus_master: pci.read(0x14) as usize & 0xFFFFFFF0,
            irq: pci.read(0x3C) as u8 & 0xF,
            bdl: memory::alloc(32 * mem::size_of::<BD>()) as *mut BD,
        };

        debug!(" + AC97 on: {:X}, {:X}, IRQ: {:X}\n", module.audio, module.bus_master, module.irq);

        let mut po_bdbar = PhysAddr::new(Pio::<u32>::new(module.bus_master as u16 + 0x10));
        po_bdbar.write(module.bdl as u32);

        module
    }
}
