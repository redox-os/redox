use alloc::boxed::Box;

use arch::context::context_switch;
use arch::memory;

use core::{cmp, mem};

use common::time::{self, Duration};

use drivers::pci::config::PciConfig;
use drivers::io::{Io, Mmio, Pio, PhysAddr};

use fs::{KScheme, Resource, Url};

use syscall;

#[repr(packed)]
struct Bd {
    ptr: PhysAddr<Mmio<u32>>,
    samples: Mmio<u32>,
}

struct Ac97Resource {
    audio: usize,
    bus_master: usize,
    bdl: *mut Bd,
}

impl Resource for Ac97Resource {
    fn dup(&self) -> syscall::Result<Box<Resource>> {
        Ok(box Ac97Resource {
            audio: self.audio,
            bus_master: self.bus_master,
            bdl: self.bdl
        })
    }

    fn path(&self, buf: &mut [u8]) -> syscall::Result <usize> {
        let path = b"audio:";

        let mut i = 0;
        while i < buf.len() && i < path.len() {
            buf[i] = path[i];
            i += 1;
        }

        Ok(i)
    }

    fn write(&mut self, buf: &[u8]) -> syscall::Result<usize> {
        unsafe {
            let audio = self.audio as u16;

            let mut master_volume = Pio::<u16>::new(audio + 2);
            let mut pcm_volume = Pio::<u16>::new(audio + 0x18);

            debugln!("MASTER {:X} PCM {:X}", master_volume.read(), pcm_volume.read());

            master_volume.write(0);
            pcm_volume.write(0x808);

            debugln!("MASTER {:X} PCM {:X}", master_volume.read(), pcm_volume.read());

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

                    {
                        let contexts = &mut *::env().contexts.get();
                        if let Ok(mut current) = contexts.current_mut() {
                            current.wake = Some(Duration::monotonic() + Duration::new(0, 10 * time::NANOS_PER_MILLI));
                            current.block("AC97 sleep 1");
                        }
                    }

                    context_switch();
                }

                debugln!("AC97 {} / {}: {} / {}",
                       po_civ.read(),
                       lvi as usize,
                       position,
                       buf.len());

                let bytes = cmp::min(65534 * 2, (buf.len() - position + 1));
                let samples = bytes / 2;

                let mut phys_buf = buf.as_ptr() as usize;
                {
                    let contexts = &mut *::env().contexts.get();
                    if let Ok(current) = contexts.current() {
                        if let Ok(phys) = current.translate(buf.as_ptr().offset(position as isize) as usize, bytes) {
                            debugln!("logical {:#X} -> physical {:#X}", &(buf.as_ptr() as usize), &phys);
                            phys_buf = phys;
                        }
                    }
                }

                (*self.bdl.offset(lvi as isize)).ptr.write(phys_buf as u32);
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

                {
                    let contexts = &mut *::env().contexts.get();
                    if let Ok(mut current) = contexts.current_mut() {
                        current.wake = Some(Duration::monotonic() + Duration::new(0, 10 * time::NANOS_PER_MILLI));
                        current.block("AC97 sleep 2");
                    }
                }

                context_switch();
            }

            debug!("AC97 Finished {} / {}\n", po_civ.read(), lvi);
        }

        Ok(buf.len())
    }
}

pub struct Ac97 {
    audio: usize,
    bus_master: usize,
    irq: u8,
    bdl: *mut Bd,
}

impl KScheme for Ac97 {
    fn scheme(&self) -> &str {
        "audio"
    }

    fn open(&mut self, _: Url, _: usize) -> syscall::Result<Box<Resource>> {
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
            bdl: memory::alloc(32 * mem::size_of::<Bd>()) as *mut Bd,
        };

        syslog_info!(" + AC97 on: {:X}, {:X}, IRQ: {:X}", module.audio, module.bus_master, module.irq);

        let mut po_bdbar = PhysAddr::new(Pio::<u32>::new(module.bus_master as u16 + 0x10));
        po_bdbar.write(module.bdl as u32);

        module
    }
}
