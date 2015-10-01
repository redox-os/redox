use core::ptr::{read, write};

use common::memory::*;
use common::scheduler::*;

use drivers::pciconfig::*;

use programs::common::*;

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
    bdlpu: u32
}

#[repr(packed)]
struct BD {
    addr: u32,
    addru: u32,
    len: u32,
    ioc: u32
}

struct IntelHDAResource {
    base: usize
}

impl Resource for IntelHDAResource {
    fn url(&self) -> URL {
        return URL::from_str("hda://");
    }

    fn stat(&self) -> ResourceType {
        return ResourceType::File;
    }

    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        return Option::None;
    }

    fn write(&mut self, buf: &[u8]) -> Option<usize> {
        unsafe {
            d("Write HDA");

            let gcap = (self.base) as *mut u16;
            d(" GCAP ");
            dh(read(gcap) as usize);

            let iss = (read(gcap) as usize >> 12) & 0b1111;
            d(" ISS ");
            dd(iss);

            let oss = (read(gcap) as usize >> 8) & 0b1111;
            d(" OSS ");
            dd(oss);

            let bss = (read(gcap) as usize >> 3) & 0b11111;
            d(" BSS ");
            dd(bss);

            dl();

            let stream = &mut *((self.base + 0x80 + iss * 0x20) as *mut Stream);

            d("Output Stream");

            d(" SizeOf ");
            dd(size_of::<Stream>());

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

            d(" Interrupt ");
            dh(stream.interrupt as usize);

            stream.control = 1 << 4 as u8;

            d(" Control ");
            dh(stream.control as usize);

            d(" Status ");
            dh(stream.status as usize);

            stream.format = 0b0000000000010001;

            d(" Format ");
            dh(stream.format as usize);

            let bd_addr = alloc(buf.len());
            let bd_size = alloc_size(bd_addr);

            ::memset(bd_addr as *mut u8, 0, bd_size);
            ::memcpy(bd_addr as *mut u8, buf.as_ptr(), buf.len());

            let bdl = alloc(2 * size_of::<BD>()) as *mut BD;
            write(bdl, BD {
                addr: bd_addr as u32,
                addru: 0,
                len: bd_size as u32,
                ioc: 1
            });
            write(bdl.offset(1), BD {
                addr: bd_addr as u32,
                addru: 0,
                len: bd_size as u32,
                ioc: 1
            });

            stream.bdlpl = bdl as u32;

            stream.cbl = (bd_size * 2) as u32;

            d(" CBL ");
            dd(stream.cbl as usize);

            stream.lvi = 1;
            d(" LVI ");
            dd(stream.lvi as usize);

            stream.interrupt = 1 << 2 | 1 << 1;

            d(" Interrupt ");
            dh(stream.interrupt as usize);

            dl();

            loop {
                d(" Interrupt ");
                dh(stream.interrupt as usize);

                d(" Control ");
                dh(stream.control as usize);

                d(" Status ");
                dh(stream.status as usize);

                d(" LPIB ");
                dd(stream.lpib as usize);
                dl();

                if stream.status & 4 == 4 {
                    break;
                }
                Duration::new(0, 10*NANOS_PER_MILLI).sleep();
            }

            d("Finished\n");
            stream.interrupt = 0;
            /*
            stream.control = 0;
            stream.status = 0;
            stream.cbl = 0;
            stream.lvi = 0;
            stream.bdlpl = 0;
            unalloc(bd_addr);
            unalloc(bdl as usize);
            */

            return Option::Some(buf.len());
        }
    }

    fn seek(&mut self, pos: ResourceSeek) -> Option<usize> {
        return Option::None;
    }

    fn flush(&mut self) -> bool {
        return false;
    }
}

pub struct IntelHDA {
    pub pci: PCIConfig,
    pub base: usize,
    pub memory_mapped: bool,
    pub irq: u8
}

impl SessionItem for IntelHDA {
    fn scheme(&self) -> String {
        return "hda".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource> {
        return box IntelHDAResource {
            base: self.base
        };
    }

    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            //d("HDA IRQ\n");
        }
    }

    fn on_poll(&mut self) {
    }
}

impl IntelHDA {
    pub unsafe fn init(&mut self) {
        d("Intel HDA on: ");
        dh(self.base);
        if self.memory_mapped {
            d(" memory mapped");
        } else {
            d(" port mapped");
        }
        d(", IRQ: ");
        dbh(self.irq);

        let pci = &mut self.pci;

        pci.flag(4, 4, true); // Bus mastering

        let gcap = (self.base) as *mut u16;
        let gctl = (self.base + 0x8) as *mut u32;
        let statests = (self.base + 0xE) as *mut u16;

        let corb = (self.base + 0x40) as *mut u32;
        let corbwp = (self.base + 0x48) as *mut u16;
        let corbrp = (self.base + 0x4A) as *mut u16;
        let corbctl = (self.base + 0x4C) as *mut u8;
        let corbsts = (self.base + 0x4D) as *mut u8;
        let corbsize = (self.base + 0x4E) as *mut u8;

        let rirb = (self.base + 0x50) as *mut u32;
        let rirbwp = (self.base + 0x58) as *mut u16;
        let rintcnt = (self.base + 0x5A) as *mut u16;
        let rirbctl = (self.base + 0x5C) as *mut u8;
        let rirbsts = (self.base + 0x5D) as *mut u8;
        let rirbsize = (self.base + 0x5E) as *mut u8;

        d(" GCAP ");
        dh(read(gcap) as usize);

        let iss = (read(gcap) as usize >> 12) & 0b1111;
        d(" ISS ");
        dd(iss);

        let oss = (read(gcap) as usize >> 8) & 0b1111;
        d(" OSS ");
        dd(oss);

        let bss = (read(gcap) as usize >> 3) & 0b11111;
        d(" BSS ");
        dd(bss);

        d(" GCTL ");
        dh(read(gctl) as usize);

        write(gctl, 0);
        loop {
            if read(gctl) & 1 == 0 {
                break;
            }
        }

        d(" GCTL ");
        dh(read(gctl) as usize);

        write(gctl, 1);
        loop {
            if read(gctl) & 1 == 1 {
                break;
            }
        }

        let disable = start_ints();
        Duration::new(0, 10*NANOS_PER_MILLI).sleep();
        end_ints(disable);

        d(" GCTL ");
        dh(read(gctl) as usize);

        d(" STATESTS ");
        dh(read(statests) as usize);

        let corb_ptr = alloc(256 * 4) as *mut u32;
        {
            write(corbctl, 0);
            loop {
                if read(corbctl) & 1 << 1 == 0 {
                    break;
                }
            }
            d(" CORBCTL ");
            dh(read(corbctl) as usize);

            write(corb, corb_ptr as u32);
            write(corbsize, 0b10);
            write(corbrp, 1 << 15);
            loop {
                if read(corbrp) == 1 << 15 {
                    break;
                }
            }
            write(corbrp, 0);
            loop {
                if read(corbrp) == 0 {
                    break;
                }
            }
            write(corbwp, 0);

            write(corbctl, 1 << 1);
            loop {
                if read(corbctl) & 1 << 1 == 1 << 1 {
                    break;
                }
            }
            d(" CORBCTL ");
            dh(read(corbctl) as usize);
        }

        let rirb_ptr = alloc(256 * 8) as *mut u64;
        {
            write(rirbctl, 0);
            loop {
                if read(rirbctl) & 1 << 1 == 0 {
                    break;
                }
            }
            d(" RIRBCTL ");
            dh(read(rirbctl) as usize);

            write(rirb, rirb_ptr as u32);
            write(rirbsize, 0b10);
            write(rirbwp, 1 << 15);
            write(rintcnt, 0xFF);

            write(rirbctl, 1 << 1);
            loop {
                if read(rirbctl) & 1 << 1 == 1 << 1 {
                    break;
                }
            }
            d(" RIRBCTL ");
            dh(read(rirbctl) as usize);
        }

        dl();

        let cmd = |command: u32| -> u64 {
            let corb_i = (read(corbwp) + 1) & 0xFF;
            let rirb_i = (read(rirbwp) + 1) & 0xFF;

            /*
            d("CORB ");
            dd(corb_i as usize);
            d(" RIRB ");
            dd(rirb_i as usize);
            dl();
            */

            write(corb_ptr.offset(corb_i as isize), command);
            write(corbwp, corb_i);

            loop {
                if read(rirbwp) == rirb_i {
                    break;
                }
            }

            return read(rirb_ptr.offset(rirb_i as isize));
        };

        let mut output_stream_id = 1;

        let root_nodes_packed = cmd(0xF0004);
        let root_nodes_start = (root_nodes_packed >> 16) as u32;
        let root_nodes_length = (root_nodes_packed & 0xFFFF) as u32;

        d("Root Sub-Nodes ");
        dd(root_nodes_start as usize);
        d(" ");
        dd(root_nodes_length as usize);
        dl();

        for fg_node in root_nodes_start..root_nodes_start + root_nodes_length {
            d("  Function Group ");
            dd(fg_node as usize);
            dl();

            let fg_type = cmd(fg_node << 20 | 0xF0005);

            d("    Type ");
            dh(fg_type as usize);
            dl();

            let fg_nodes_packed = cmd(fg_node << 20 | 0xF0004);
            let fg_nodes_start = (fg_nodes_packed >> 16) as u32;
            let fg_nodes_length = (fg_nodes_packed & 0xFFFF) as u32;

            d("    Sub-Nodes ");
            dd(fg_nodes_start as usize);
            d(" ");
            dd(fg_nodes_length as usize);
            dl();

            for w_node in fg_nodes_start..fg_nodes_start + fg_nodes_length {
                d("      Widget ");
                dh(w_node as usize);
                dl();

                let w_caps = cmd(w_node << 20 | 0xF0009);

                d("        Capabilities ");
                dh(w_caps as usize);
                dl();

                match w_caps >> 20 {
                    0 => {
                        d("        Type: Output\n");

                        d("        Sample Rate and Bits ");
                        dh(cmd(w_node << 20 | 0xF000A) as usize);
                        dl();

                        d("        Sample Format ");
                        dh(cmd(w_node << 20 | 0xF000B) as usize);
                        dl();

                        d("        Output Stream (Before) ");
                        dh(cmd(w_node << 20 | 0xF0600) as usize);
                        dl();

                        cmd(w_node << 20 | 0x70600 | (output_stream_id as u32) << 4);

                        d("        Output Stream (After) ");
                        dh(cmd(w_node << 20 | 0xF0600) as usize);
                        dl();

                        d("        Format (Before) ");
                        dh(cmd(w_node << 20 | 0xA0000) as usize);
                        dl();

                        cmd(w_node << 20 | 0x20000 | 0b0000000000010001);

                        d("        Format (After) ");
                        dh(cmd(w_node << 20 | 0xA0000) as usize);
                        dl();

                        /*
                        d("        Amplifier Gain/Mute (Before) ");
                        dh(cmd(w_node << 20 | 0xB0000 | 1 << 15 | 1 << 13) as usize);
                        d(" ");
                        dh(cmd(w_node << 20 | 0xB0000 | 1 << 15) as usize);
                        dl();

                        cmd(w_node << 20 | 0x30000 | 1 << 15 | 1 << 13 | 1 << 12 | 0b111111);

                        d("        Amplifier Gain/Mute (After) ");
                        dh(cmd(w_node << 20 | 0xB0000 | 1 << 15 | 1 << 13) as usize);
                        d(" ");
                        dh(cmd(w_node << 20 | 0xB0000 | 1 << 15) as usize);
                        dl();
                        */

                        output_stream_id += 1;
                    },
                    1 => {
                        d("        Type: Input\n");

                        d("        Input Stream ");
                        dh(cmd(w_node << 20 | 0xF0600) as usize);
                        dl();
                    },
                    2 => d("        Type: Mixer\n"),
                    3 => d("        Type: Selector\n"),
                    4 => {
                        d("        Type: Pin\n");

                        d("        Pin Capabilities ");
                        dh(cmd(w_node << 20 | 0xF000C) as usize);
                        dl();

                        d("        Pin Control (Before) ");
                        dh(cmd(w_node << 20 | 0xF0700) as usize);
                        dl();

                        cmd(w_node << 20 | 0x70700 | 0b11100101);

                        d("        Pin Control (After) ");
                        dh(cmd(w_node << 20 | 0xF0700) as usize);
                        dl();

                        /*
                        d("        Pin EAPD/BTL (Before) ");
                        dh(cmd(w_node << 20 | 0xF0C00) as usize);
                        dl();

                        cmd(w_node << 20 | 0x70C00 | 1 << 1);

                        d("        Pin EAPD/BPL (After) ");
                        dh(cmd(w_node << 20 | 0xF0C00) as usize);
                        dl();

                        d("        Amplifier Gain/Mute (Before) ");
                        dh(cmd(w_node << 20 | 0xB0000 | 1 << 15 | 1 << 13) as usize);
                        d(" ");
                        dh(cmd(w_node << 20 | 0xB0000 | 1 << 15) as usize);
                        dl();

                        cmd(w_node << 20 | 0x30000 | 1 << 15 | 1 << 13 | 1 << 12 | 0b111111);

                        d("        Amplifier Gain/Mute (After) ");
                        dh(cmd(w_node << 20 | 0xB0000 | 1 << 15 | 1 << 13) as usize);
                        d(" ");
                        dh(cmd(w_node << 20 | 0xB0000 | 1 << 15) as usize);
                        dl();
                        */
                    },
                    5 => d("        Type: Power\n"),
                    6 => d("        Type: Volume\n"),
                    7 => d("        Type: Beep Generator\n"),
                    _ => {
                        d("        Type: Unknown\n");
                    }
                }
            }
        }
    }
}
