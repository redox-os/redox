use alloc::boxed::Box;

use arch::memory::Memory;

use core::ptr;

use drivers::pci::config::PciConfig;

use common::time;

use fs::{KScheme, Resource, Url};

use syscall;
use syscall::TimeSpec;

#[repr(packed)]
struct Stream {
    interrupt: u8,
    reserved: u8,
    control: u8,
    status: u8,
    // RO Position of link in buffer
    lpib: u32,
    // RW length of buffer
    cbl: u32,
    // RW last valid buffer description
    lvi: u16,
    reserved_2: u16,
    // RO maximum number of fetchable bytes
    fifos: u16,
    // format
    format: u16,
    reserved_3: u32,
    // pointer to buffer descriptor list
    bdlpl: u32,
    bdlpu: u32,
}

#[repr(packed)]
struct BD {
    addr: u32,
    addru: u32,
    len: u32,
    ioc: u32,
}

struct IntelHdaResource {
    base: usize,
}

impl Resource for IntelHdaResource {
    fn dup(&self) -> syscall::Result<Box<Resource>> {
        Ok(box IntelHdaResource { base: self.base })
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
            debug!("Write HDA");

            let gcap = (self.base) as *mut u16;

            let iss = (ptr::read(gcap) as usize >> 12) & 0b1111;

            let stream = &mut *((self.base + 0x80 + iss * 0x20) as *mut Stream);

            stream.interrupt = 1;
            loop {
                if stream.interrupt & 1 == 1 {
                    break;
                }
            }

            stream.interrupt = 0;
            loop {
                if stream.interrupt & 1 == 0 {
                    break;
                }
            }

            stream.control = 1 << 4 as u8;

            stream.format = 0b0000000000010001;

            let mut bd_addr = try!(Memory::<u8>::new(buf.len()));
            let bd_size = bd_addr.len();

            ::memset(bd_addr.as_mut_ptr(), 0, bd_size);
            ::memcpy(bd_addr.as_mut_ptr(), buf.as_ptr(), buf.len());

            let mut bdl = try!(Memory::<BD>::new(2));
            bdl.write(0, BD {
                addr: bd_addr.address() as u32,
                addru: 0,
                len: bd_size as u32,
                ioc: 1,
            });
            bdl.write(1, BD {
                addr: bd_addr.address() as u32,
                addru: 0,
                len: bd_size as u32,
                ioc: 1,
            });

            stream.bdlpl = bdl.address() as u32;

            stream.cbl = (bd_size * 2) as u32;

            stream.lvi = 1;

            stream.interrupt = 1 << 2 | 1 << 1;

            loop {
                if stream.status & 4 == 4 {
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
                try!(syscall::time::nanosleep(&req, &mut rem));
            }

            stream.interrupt = 0;
            // stream.control = 0;
            // stream.status = 0;
            // stream.cbl = 0;
            // stream.lvi = 0;
            // stream.bdlpl = 0;
            // memory::unalloc(bd_addr);
            // memory::unalloc(bdl as usize);
            //

            Ok(buf.len())
        }
    }
}

pub struct IntelHda {
    pub pci: PciConfig,
    pub base: usize,
    pub memory_mapped: bool,
    pub irq: u8,
}

impl KScheme for IntelHda {
    fn scheme(&self) -> &str {
        "hda"
    }

    fn open(&mut self, _: Url, _: usize) -> syscall::Result<Box<Resource>> {
        Ok(box IntelHdaResource { base: self.base })
    }

    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            // d("HDA IRQ\n");
        }
    }
}

impl IntelHda {
    pub unsafe fn new(mut pci: PciConfig) -> Box<IntelHda> {
        let base = pci.read(0x10) as usize;
        let mut module = box IntelHda {
            pci: pci,
            base: base & 0xFFFFFFF0,
            memory_mapped: base & 1 == 0,
            irq: pci.read(0x3C) as u8 & 0xF,
        };
        module.init();
        module
    }

    pub unsafe fn init(&mut self) {
        debugln!(" + Intel HDA on: {:X}, IRQ {:X}", self.base, self.irq);

        return;
        // let pci = &mut self.pci;
        //
        // pci.flag(4, 4, true); // Bus mastering
        //
        // let gcap = (self.base) as *mut u16;
        // let gctl = (self.base + 0x8) as *mut u32;
        // let statests = (self.base + 0xE) as *mut u16;
        //
        // let corb = (self.base + 0x40) as *mut u32;
        // let corbwp = (self.base + 0x48) as *mut u16;
        // let corbrp = (self.base + 0x4A) as *mut u16;
        // let corbctl = (self.base + 0x4C) as *mut u8;
        // let corbsize = (self.base + 0x4E) as *mut u8;
        //
        // let rirb = (self.base + 0x50) as *mut u32;
        // let rirbwp = (self.base + 0x58) as *mut u16;
        // let rintcnt = (self.base + 0x5A) as *mut u16;
        // let rirbctl = (self.base + 0x5C) as *mut u8;
        // let rirbsize = (self.base + 0x5E) as *mut u8;
        //
        // let iss = (ptr::read(gcap) as usize >> 12) & 0b1111;
        //
        // let oss = (ptr::read(gcap) as usize >> 8) & 0b1111;
        //
        // let bss = (ptr::read(gcap) as usize >> 3) & 0b11111;
        //
        // ptr::write(gctl, 0);
        // loop {
        // if ptr::read(gctl) & 1 == 0 {
        // break;
        // }
        // }
        //
        // ptr::write(gctl, 1);
        // loop {
        // if ptr::read(gctl) & 1 == 1 {
        // break;
        // }
        // }
        //
        // let disable = scheduler::start_ints();
        // Duration::new(0, 10 * time::NANOS_PER_MILLI).sleep();
        // scheduler::end_ints(disable);
        //
        // let corb_ptr = memory::alloc(256 * 4) as *mut u32;
        // {
        // ptr::write(corbctl, 0);
        // loop {
        // if ptr::read(corbctl) & 1 << 1 == 0 {
        // break;
        // }
        // }
        //
        // ptr::write(corb, corb_ptr as u32);
        // ptr::write(corbsize, 0b10);
        // ptr::write(corbrp, 1 << 15);
        // loop {
        // if ptr::read(corbrp) == 1 << 15 {
        // break;
        // }
        // }
        // ptr::write(corbrp, 0);
        // loop {
        // if ptr::read(corbrp) == 0 {
        // break;
        // }
        // }
        // ptr::write(corbwp, 0);
        //
        // ptr::write(corbctl, 1 << 1);
        // loop {
        // if ptr::read(corbctl) & 1 << 1 == 1 << 1 {
        // break;
        // }
        // }
        // }
        //
        // let rirb_ptr = memory::alloc(256 * 8) as *mut u64;
        // {
        // ptr::write(rirbctl, 0);
        // loop {
        // if ptr::read(rirbctl) & 1 << 1 == 0 {
        // break;
        // }
        // }
        //
        // ptr::write(rirb, rirb_ptr as u32);
        // ptr::write(rirbsize, 0b10);
        // ptr::write(rirbwp, 1 << 15);
        // ptr::write(rintcnt, 0xFF);
        //
        // ptr::write(rirbctl, 1 << 1);
        // loop {
        // if ptr::read(rirbctl) & 1 << 1 == 1 << 1 {
        // break;
        // }
        // }
        // }
        //
        // let cmd = |command: u32| -> u64 {
        // let corb_i = (ptr::read(corbwp) + 1) & 0xFF;
        // let rirb_i = (ptr::read(rirbwp) + 1) & 0xFF;
        //
        //
        // d("CORB ");
        // dd(corb_i as usize);
        // d(" RIRB ");
        // dd(rirb_i as usize);
        // dl();
        //
        //
        // ptr::write(corb_ptr.offset(corb_i as isize), command);
        // ptr::write(corbwp, corb_i);
        //
        // loop {
        // if ptr::read(rirbwp) == rirb_i {
        // break;
        // }
        // }
        //
        // ptr::read(rirb_ptr.offset(rirb_i as isize))
        // };
        //
        // let mut output_stream_id = 1;
        //
        // let root_nodes_packed = cmd(0xF0004);
        // let root_nodes_start = (root_nodes_packed >> 16) as u32;
        // let root_nodes_length = (root_nodes_packed & 0xFFFF) as u32;
        //
        // debug!("Root Sub-Nodes ");
        // debug::dd(root_nodes_start as usize);
        // debug!(" ");
        // debug::dd(root_nodes_length as usize);
        // debug::dl();
        //
        // for fg_node in root_nodes_start..root_nodes_start + root_nodes_length {
        // debug!("  Function Group ");
        // debug::dd(fg_node as usize);
        // debug::dl();
        //
        // let fg_type = cmd(fg_node << 20 | 0xF0005);
        //
        // debug!("    Type ");
        // debug::dh(fg_type as usize);
        // debug::dl();
        //
        // let fg_nodes_packed = cmd(fg_node << 20 | 0xF0004);
        // let fg_nodes_start = (fg_nodes_packed >> 16) as u32;
        // let fg_nodes_length = (fg_nodes_packed & 0xFFFF) as u32;
        //
        // debug!("    Sub-Nodes ");
        // debug::dd(fg_nodes_start as usize);
        // debug!(" ");
        // debug::dd(fg_nodes_length as usize);
        // debug::dl();
        //
        // for w_node in fg_nodes_start..fg_nodes_start + fg_nodes_length {
        // debug!("      Widget ");
        // debug::dh(w_node as usize);
        // debug::dl();
        //
        // let w_caps = cmd(w_node << 20 | 0xF0009);
        //
        // debug!("        Capabilities ");
        // debug::dh(w_caps as usize);
        // debug::dl();
        //
        // match w_caps >> 20 {
        // 0 => {
        // debug!("        Type: Output\n");
        //
        // debug!("        Sample Rate and Bits ");
        // debug::dh(cmd(w_node << 20 | 0xF000A) as usize);
        // debug::dl();
        //
        // debug!("        Sample Format ");
        // debug::dh(cmd(w_node << 20 | 0xF000B) as usize);
        // debug::dl();
        //
        // debug!("        Output Stream (Before) ");
        // debug::dh(cmd(w_node << 20 | 0xF0600) as usize);
        // debug::dl();
        //
        // cmd(w_node << 20 | 0x70600 | (output_stream_id as u32) << 4);
        //
        // debug!("        Output Stream (After) ");
        // debug::dh(cmd(w_node << 20 | 0xF0600) as usize);
        // debug::dl();
        //
        // debug!("        Format (Before) ");
        // debug::dh(cmd(w_node << 20 | 0xA0000) as usize);
        // debug::dl();
        //
        // cmd(w_node << 20 | 0x20000 | 0b0000000000010001);
        //
        // debug!("        Format (After) ");
        // debug::dh(cmd(w_node << 20 | 0xA0000) as usize);
        // debug::dl();
        //
        //
        // debug!("        Amplifier Gain/Mute (Before) ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15 | 1 << 13) as usize);
        // debug!(" ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15) as usize);
        // debug::dl();
        //
        // cmd(w_node << 20 | 0x30000 | 1 << 15 | 1 << 13 | 1 << 12 | 0b111111);
        //
        // debug!("        Amplifier Gain/Mute (After) ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15 | 1 << 13) as usize);
        // debug!(" ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15) as usize);
        // debug::dl();
        //
        //
        // output_stream_id += 1;
        // }
        // 1 => {
        // debug!("        Type: Input\n");
        //
        // debug!("        Input Stream ");
        // debug::dh(cmd(w_node << 20 | 0xF0600) as usize);
        // debug::dl();
        // }
        // 2 => debug!("        Type: Mixer\n"),
        // 3 => debug!("        Type: Selector\n"),
        // 4 => {
        // debug!("        Type: Pin\n");
        //
        // debug!("        Pin Capabilities ");
        // debug::dh(cmd(w_node << 20 | 0xF000C) as usize);
        // debug::dl();
        //
        // debug!("        Pin Control (Before) ");
        // debug::dh(cmd(w_node << 20 | 0xF0700) as usize);
        // debug::dl();
        //
        // cmd(w_node << 20 | 0x70700 | 0b11100101);
        //
        // debug!("        Pin Control (After) ");
        // debug::dh(cmd(w_node << 20 | 0xF0700) as usize);
        // debug::dl();
        //
        //
        // debug!("        Pin EAPD/BTL (Before) ");
        // debug::dh(cmd(w_node << 20 | 0xF0C00) as usize);
        // debug::dl();
        //
        // cmd(w_node << 20 | 0x70C00 | 1 << 1);
        //
        // debug!("        Pin EAPD/BPL (After) ");
        // debug::dh(cmd(w_node << 20 | 0xF0C00) as usize);
        // debug::dl();
        //
        // debug!("        Amplifier Gain/Mute (Before) ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15 | 1 << 13) as usize);
        // debug!(" ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15) as usize);
        // debug::dl();
        //
        // cmd(w_node << 20 | 0x30000 | 1 << 15 | 1 << 13 | 1 << 12 | 0b111111);
        //
        // debug!("        Amplifier Gain/Mute (After) ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15 | 1 << 13) as usize);
        // debug!(" ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15) as usize);
        // debug::dl();
        //
        // }
        // 5 => debug!("        Type: Power\n"),
        // 6 => debug!("        Type: Volume\n"),
        // 7 => debug!("        Type: Beep Generator\n"),
        // _ => {
        // debug!("        Type: Unknown\n");
        // }
        // }
        // }
        // }
        //
    }
}
