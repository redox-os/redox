use alloc::boxed::Box;

use core::{ptr, mem};

use drivers::pci::config::PciConfig;

use common::debug;
use common::memory;
use common::time::{self, Duration};

use schemes::{Result, KScheme, Resource, ResourceSeek, Url};

use syscall::{Error, EBADF};

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

struct IntelHDAResource {
    base: usize,
}

impl Resource for IntelHDAResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(box IntelHDAResource { base: self.base })
    }

    fn url(&self) -> Url {
        Url::from_str("audio:")
    }

    fn read(&mut self, _: &mut [u8]) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        unsafe {
            debug::d("Write HDA");

            let gcap = (self.base) as *mut u16;
            debug::d(" GCAP ");
            debug::dh(ptr::read(gcap) as usize);

            let iss = (ptr::read(gcap) as usize >> 12) & 0b1111;
            debug::d(" ISS ");
            debug::dd(iss);

            let oss = (ptr::read(gcap) as usize >> 8) & 0b1111;
            debug::d(" OSS ");
            debug::dd(oss);

            let bss = (ptr::read(gcap) as usize >> 3) & 0b11111;
            debug::d(" BSS ");
            debug::dd(bss);

            debug::dl();

            let stream = &mut *((self.base + 0x80 + iss * 0x20) as *mut Stream);

            debug::d("Output Stream");

            debug::d(" SizeOf ");
            debug::dd(mem::size_of::<Stream>());

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

            debug::d(" Interrupt ");
            debug::dh(stream.interrupt as usize);

            stream.control = 1 << 4 as u8;

            debug::d(" Control ");
            debug::dh(stream.control as usize);

            debug::d(" Status ");
            debug::dh(stream.status as usize);

            stream.format = 0b0000000000010001;

            debug::d(" Format ");
            debug::dh(stream.format as usize);

            let bd_addr = memory::alloc(buf.len());
            let bd_size = memory::alloc_size(bd_addr);

            ::memset(bd_addr as *mut u8, 0, bd_size);
            ::memcpy(bd_addr as *mut u8, buf.as_ptr(), buf.len());

            let bdl = memory::alloc(2 * mem::size_of::<BD>()) as *mut BD;
            ptr::write(bdl,
                       BD {
                           addr: bd_addr as u32,
                           addru: 0,
                           len: bd_size as u32,
                           ioc: 1,
                       });
            ptr::write(bdl.offset(1),
                       BD {
                           addr: bd_addr as u32,
                           addru: 0,
                           len: bd_size as u32,
                           ioc: 1,
                       });

            stream.bdlpl = bdl as u32;

            stream.cbl = (bd_size * 2) as u32;

            debug::d(" CBL ");
            debug::dd(stream.cbl as usize);

            stream.lvi = 1;
            debug::d(" LVI ");
            debug::dd(stream.lvi as usize);

            stream.interrupt = 1 << 2 | 1 << 1;

            debug::d(" Interrupt ");
            debug::dh(stream.interrupt as usize);

            debug::dl();

            loop {
                debug::d(" Interrupt ");
                debug::dh(stream.interrupt as usize);

                debug::d(" Control ");
                debug::dh(stream.control as usize);

                debug::d(" Status ");
                debug::dh(stream.status as usize);

                debug::d(" LPIB ");
                debug::dd(stream.lpib as usize);
                debug::dl();

                if stream.status & 4 == 4 {
                    break;
                }
                Duration::new(0, 10 * time::NANOS_PER_MILLI).sleep();
            }

            debug::d("Finished\n");
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

    fn seek(&mut self, _: ResourceSeek) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    fn sync(&mut self) -> Result<()> {
        Err(Error::new(EBADF))
    }
}

pub struct IntelHDA {
    pub pci: PciConfig,
    pub base: usize,
    pub memory_mapped: bool,
    pub irq: u8,
}

impl KScheme for IntelHDA {
    fn scheme(&self) -> &str {
        "hda"
    }

    fn open(&mut self, _: &Url, _: usize) -> Result<Box<Resource>> {
        Ok(box IntelHDAResource { base: self.base })
    }

    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            // d("HDA IRQ\n");
        }
    }

    fn on_poll(&mut self) {}
}

impl IntelHDA {
    pub unsafe fn init(&mut self) {
        debug::d("Intel HDA on: ");
        debug::dh(self.base);
        if self.memory_mapped {
            debug::d(" memory mapped");
        } else {
            debug::d(" port mapped");
        }
        debug::d(", IRQ: ");
        debug::dbh(self.irq);
        debug::dl();

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
        // debug::d(" GCAP ");
        // debug::dh(ptr::read(gcap) as usize);
        //
        // let iss = (ptr::read(gcap) as usize >> 12) & 0b1111;
        // debug::d(" ISS ");
        // debug::dd(iss);
        //
        // let oss = (ptr::read(gcap) as usize >> 8) & 0b1111;
        // debug::d(" OSS ");
        // debug::dd(oss);
        //
        // let bss = (ptr::read(gcap) as usize >> 3) & 0b11111;
        // debug::d(" BSS ");
        // debug::dd(bss);
        //
        // debug::d(" GCTL ");
        // debug::dh(ptr::read(gctl) as usize);
        //
        // ptr::write(gctl, 0);
        // loop {
        // if ptr::read(gctl) & 1 == 0 {
        // break;
        // }
        // }
        //
        // debug::d(" GCTL ");
        // debug::dh(ptr::read(gctl) as usize);
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
        // debug::d(" GCTL ");
        // debug::dh(ptr::read(gctl) as usize);
        //
        // debug::d(" STATESTS ");
        // debug::dh(ptr::read(statests) as usize);
        //
        // let corb_ptr = memory::alloc(256 * 4) as *mut u32;
        // {
        // ptr::write(corbctl, 0);
        // loop {
        // if ptr::read(corbctl) & 1 << 1 == 0 {
        // break;
        // }
        // }
        // debug::d(" CORBCTL ");
        // debug::dh(ptr::read(corbctl) as usize);
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
        // debug::d(" CORBCTL ");
        // debug::dh(ptr::read(corbctl) as usize);
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
        // debug::d(" RIRBCTL ");
        // debug::dh(ptr::read(rirbctl) as usize);
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
        // debug::d(" RIRBCTL ");
        // debug::dh(ptr::read(rirbctl) as usize);
        // }
        //
        // debug::dl();
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
        // debug::d("Root Sub-Nodes ");
        // debug::dd(root_nodes_start as usize);
        // debug::d(" ");
        // debug::dd(root_nodes_length as usize);
        // debug::dl();
        //
        // for fg_node in root_nodes_start..root_nodes_start + root_nodes_length {
        // debug::d("  Function Group ");
        // debug::dd(fg_node as usize);
        // debug::dl();
        //
        // let fg_type = cmd(fg_node << 20 | 0xF0005);
        //
        // debug::d("    Type ");
        // debug::dh(fg_type as usize);
        // debug::dl();
        //
        // let fg_nodes_packed = cmd(fg_node << 20 | 0xF0004);
        // let fg_nodes_start = (fg_nodes_packed >> 16) as u32;
        // let fg_nodes_length = (fg_nodes_packed & 0xFFFF) as u32;
        //
        // debug::d("    Sub-Nodes ");
        // debug::dd(fg_nodes_start as usize);
        // debug::d(" ");
        // debug::dd(fg_nodes_length as usize);
        // debug::dl();
        //
        // for w_node in fg_nodes_start..fg_nodes_start + fg_nodes_length {
        // debug::d("      Widget ");
        // debug::dh(w_node as usize);
        // debug::dl();
        //
        // let w_caps = cmd(w_node << 20 | 0xF0009);
        //
        // debug::d("        Capabilities ");
        // debug::dh(w_caps as usize);
        // debug::dl();
        //
        // match w_caps >> 20 {
        // 0 => {
        // debug::d("        Type: Output\n");
        //
        // debug::d("        Sample Rate and Bits ");
        // debug::dh(cmd(w_node << 20 | 0xF000A) as usize);
        // debug::dl();
        //
        // debug::d("        Sample Format ");
        // debug::dh(cmd(w_node << 20 | 0xF000B) as usize);
        // debug::dl();
        //
        // debug::d("        Output Stream (Before) ");
        // debug::dh(cmd(w_node << 20 | 0xF0600) as usize);
        // debug::dl();
        //
        // cmd(w_node << 20 | 0x70600 | (output_stream_id as u32) << 4);
        //
        // debug::d("        Output Stream (After) ");
        // debug::dh(cmd(w_node << 20 | 0xF0600) as usize);
        // debug::dl();
        //
        // debug::d("        Format (Before) ");
        // debug::dh(cmd(w_node << 20 | 0xA0000) as usize);
        // debug::dl();
        //
        // cmd(w_node << 20 | 0x20000 | 0b0000000000010001);
        //
        // debug::d("        Format (After) ");
        // debug::dh(cmd(w_node << 20 | 0xA0000) as usize);
        // debug::dl();
        //
        //
        // debug::d("        Amplifier Gain/Mute (Before) ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15 | 1 << 13) as usize);
        // debug::d(" ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15) as usize);
        // debug::dl();
        //
        // cmd(w_node << 20 | 0x30000 | 1 << 15 | 1 << 13 | 1 << 12 | 0b111111);
        //
        // debug::d("        Amplifier Gain/Mute (After) ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15 | 1 << 13) as usize);
        // debug::d(" ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15) as usize);
        // debug::dl();
        //
        //
        // output_stream_id += 1;
        // }
        // 1 => {
        // debug::d("        Type: Input\n");
        //
        // debug::d("        Input Stream ");
        // debug::dh(cmd(w_node << 20 | 0xF0600) as usize);
        // debug::dl();
        // }
        // 2 => debug::d("        Type: Mixer\n"),
        // 3 => debug::d("        Type: Selector\n"),
        // 4 => {
        // debug::d("        Type: Pin\n");
        //
        // debug::d("        Pin Capabilities ");
        // debug::dh(cmd(w_node << 20 | 0xF000C) as usize);
        // debug::dl();
        //
        // debug::d("        Pin Control (Before) ");
        // debug::dh(cmd(w_node << 20 | 0xF0700) as usize);
        // debug::dl();
        //
        // cmd(w_node << 20 | 0x70700 | 0b11100101);
        //
        // debug::d("        Pin Control (After) ");
        // debug::dh(cmd(w_node << 20 | 0xF0700) as usize);
        // debug::dl();
        //
        //
        // debug::d("        Pin EAPD/BTL (Before) ");
        // debug::dh(cmd(w_node << 20 | 0xF0C00) as usize);
        // debug::dl();
        //
        // cmd(w_node << 20 | 0x70C00 | 1 << 1);
        //
        // debug::d("        Pin EAPD/BPL (After) ");
        // debug::dh(cmd(w_node << 20 | 0xF0C00) as usize);
        // debug::dl();
        //
        // debug::d("        Amplifier Gain/Mute (Before) ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15 | 1 << 13) as usize);
        // debug::d(" ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15) as usize);
        // debug::dl();
        //
        // cmd(w_node << 20 | 0x30000 | 1 << 15 | 1 << 13 | 1 << 12 | 0b111111);
        //
        // debug::d("        Amplifier Gain/Mute (After) ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15 | 1 << 13) as usize);
        // debug::d(" ");
        // debug::dh(cmd(w_node << 20 | 0xB0000 | 1 << 15) as usize);
        // debug::dl();
        //
        // }
        // 5 => debug::d("        Type: Power\n"),
        // 6 => debug::d("        Type: Volume\n"),
        // 7 => debug::d("        Type: Beep Generator\n"),
        // _ => {
        // debug::d("        Type: Unknown\n");
        // }
        // }
        // }
        // }
        //
    }
}
