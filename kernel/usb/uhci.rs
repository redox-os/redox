use alloc::boxed::Box;

use collections::string::ToString;
use collections::vec::Vec;

use core::intrinsics::volatile_load;
use core::{cmp, mem, ptr, slice};

use scheduler::context::{context_switch, Context};
use common::debug;
use common::event::MouseEvent;
use common::memory::{self, Memory};
use common::time::{self, Duration};

use drivers::pciconfig::PciConfig;
use drivers::pio::*;

use graphics::display::VBEMODEINFO;

use schemes::KScheme;

use super::UsbMsg;
use super::desc::*;
use super::setup::Setup;

pub struct Uhci {
    pub base: usize,
    pub irq: u8,
    pub frame_list: Memory<u32>,
}

impl KScheme for Uhci {
    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            // d("UHCI IRQ\n");
        }
    }

    fn on_poll(&mut self) {
    }
}

#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
struct Td {
    link_ptr: u32,
    ctrl_sts: u32,
    token: u32,
    buffer: u32,
}

#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
struct Qh {
    head_ptr: u32,
    element_ptr: u32,
}

impl Uhci {
    pub unsafe fn new(mut pci: PciConfig) -> Box<Self> {
        pci.flag(4, 4, true); // Bus mastering

        let mut module = box Uhci {
            base: pci.read(0x20) as usize & 0xFFFFFFF0,
            irq: pci.read(0x3C) as u8 & 0xF,
            frame_list: Memory::new(1024).unwrap(),
        };

        module.init();

        return module;
    }

    fn msg(&mut self, address: u8, endpoint: u8, msgs: &[UsbMsg]) -> usize {
        let mut tds = Vec::new();
        for msg in msgs.iter().rev() {
            let link_ptr = match tds.last() {
                Some(td) => (td as *const Td) as u32 | 4,
                None => 1
            };

            match *msg {
                UsbMsg::Setup(setup) => tds.push(Td {
                    link_ptr: link_ptr,
                    ctrl_sts: 1 << 23,
                    token: (mem::size_of::<Setup>() as u32 - 1) << 21 | (endpoint as u32) << 15 | (address as u32) << 8 | 0x2D,
                    buffer: (&*setup as *const Setup) as u32,
                }),
                UsbMsg::In(ref data) => tds.push(Td {
                    link_ptr: link_ptr,
                    ctrl_sts: 1 << 23,
                    token: ((data.len() as u32 - 1) & 0x7FF) << 21 | (endpoint as u32) << 15 | (address as u32) << 8 | 0x69,
                    buffer: data.as_ptr() as u32,
                }),
                UsbMsg::InIso(ref data) => tds.push(Td {
                    link_ptr: link_ptr,
                    ctrl_sts: 1 << 25 | 1 << 23,
                    token: ((data.len() as u32 - 1) & 0x7FF) << 21 | (endpoint as u32) << 15 | (address as u32) << 8 | 0x69,
                    buffer: data.as_ptr() as u32,
                }),
                UsbMsg::Out(ref data) => tds.push(Td {
                    link_ptr: link_ptr,
                    ctrl_sts: 1 << 23,
                    token: ((data.len() as u32 - 1) & 0x7FF) << 21 | (endpoint as u32) << 15 | (address as u32) << 8 | 0xE1,
                    buffer: data.as_ptr() as u32,
                }),
                UsbMsg::OutIso(ref data) => tds.push(Td {
                    link_ptr: link_ptr,
                    ctrl_sts: 1 << 25 | 1 << 23,
                    token: ((data.len() as u32 - 1) & 0x7FF) << 21 | (endpoint as u32) << 15 | (address as u32) << 8 | 0xE1,
                    buffer: data.as_ptr() as u32,
                })
            }
        }

        let mut count = 0;

        if ! tds.is_empty() {
            let queue_head = box Qh {
                 head_ptr: 1,
                 element_ptr: (tds.last().unwrap() as *const Td) as u32,
            };

            let frame_ptr = if tds.len() == 1 {
                (&tds[0] as *const Td) as u32
            } else {
                (&*queue_head as *const Qh) as u32 | 2
            };

            let frnum = Pio16::new(self.base as u16 + 6);
            let frame = (unsafe { frnum.read() } + 1) & 0x3FF;
            unsafe { self.frame_list.write(frame as usize, frame_ptr) };

            for td in tds.iter().rev() {
                while unsafe { volatile_load(td as *const Td).ctrl_sts } & 1 << 23 == 1 << 23 {
                    unsafe { context_switch(false) };
                }
                count += (unsafe { volatile_load(td as *const Td).ctrl_sts } & 0x7FF) as usize;
            }

            unsafe { self.frame_list.write(frame as usize, 1) };
        }

        count
    }

    fn descriptor(&mut self,
                         address: u8,
                         descriptor_type: u8,
                         descriptor_index: u8,
                         descriptor_ptr: usize,
                         descriptor_len: usize) {
        self.msg(address, 0, &[
            UsbMsg::Setup(&Setup::get_descriptor(descriptor_type, descriptor_index, 0, descriptor_len as u16)),
            UsbMsg::In(&mut unsafe { slice::from_raw_parts_mut(descriptor_ptr as *mut u8, descriptor_len as usize) }),
            UsbMsg::Out(&[])
        ]);
    }

    unsafe fn device(&mut self, address: u8) {
        self.msg(0, 0, &[
            UsbMsg::Setup(&Setup::set_address(address)),
            UsbMsg::In(&mut [])
        ]);

        let mut desc_dev = box DeviceDescriptor::default();
        self.descriptor(address,
                        DESC_DEV,
                        0,
                        (&mut *desc_dev as *mut DeviceDescriptor) as usize,
                        mem::size_of_val(&*desc_dev));
        debugln!("{:#?}", *desc_dev);

        for configuration in 0..(*desc_dev).configurations {
            let desc_cfg_len = 1023;
            let desc_cfg_buf = memory::alloc(desc_cfg_len) as *mut u8;
            for i in 0..desc_cfg_len as isize {
                ptr::write(desc_cfg_buf.offset(i), 0);
            }
            self.descriptor(address,
                            DESC_CFG,
                            configuration,
                            desc_cfg_buf as usize,
                            desc_cfg_len);

            let desc_cfg = ptr::read(desc_cfg_buf as *const ConfigDescriptor);
            debugln!("{:#?}", desc_cfg);

            let mut hid = false;

            let mut i = desc_cfg.length as isize;
            while i < desc_cfg.total_length as isize {
                let length = ptr::read(desc_cfg_buf.offset(i));
                let descriptor_type = ptr::read(desc_cfg_buf.offset(i + 1));
                match descriptor_type {
                    DESC_INT => {
                        let desc_int = ptr::read(desc_cfg_buf.offset(i) as *const InterfaceDescriptor);
                        debugln!("{:#?}", desc_int);
                    }
                    DESC_END => {
                        let desc_end = ptr::read(desc_cfg_buf.offset(i) as *const EndpointDescriptor);
                        debugln!("{:#?}", desc_end);

                        let endpoint = desc_end.address & 0xF;
                        let in_len = desc_end.max_packet_size as usize;

                        if hid {
                            let this = self as *mut Uhci;
                            Context::spawn("kuhci_hid".to_string(), box move || {
                                debugln!("Starting HID driver");

                                let in_ptr = memory::alloc(in_len) as *mut u8;

                                loop {
                                    for i in 0..in_len as isize {
                                        ptr::write(in_ptr.offset(i), 0);
                                    }

                                    if (*this).msg(address, endpoint, &[
                                        UsbMsg::InIso(&mut slice::from_raw_parts_mut(in_ptr, in_len))
                                    ]) > 0 {
                                        let buttons = ptr::read(in_ptr.offset(0) as *const u8) as usize;
                                        let x = ptr::read(in_ptr.offset(1) as *const u16) as usize;
                                        let y = ptr::read(in_ptr.offset(3) as *const u16) as usize;

                                        let mode_info = &*VBEMODEINFO;
                                        let mouse_x = (x * mode_info.xresolution as usize) / 32768;
                                        let mouse_y = (y * mode_info.yresolution as usize) / 32768;

                                        let mouse_event = MouseEvent {
                                            x: cmp::max(0, cmp::min(mode_info.xresolution as i32 - 1, mouse_x as i32)),
                                            y: cmp::max(0, cmp::min(mode_info.yresolution as i32 - 1, mouse_y as i32)),
                                            left_button: buttons & 1 == 1,
                                            middle_button: buttons & 4 == 4,
                                            right_button: buttons & 2 == 2,
                                        };
                                        ::env().events.lock().push_back(mouse_event.to_event());
                                    }

                                    Duration::new(0, 10 * time::NANOS_PER_MILLI).sleep();
                                }
                            });
                        }
                    }
                    DESC_HID => {
                        let desc_hid = &*(desc_cfg_buf.offset(i) as *const HIDDescriptor);
                        debugln!("{:#?}", desc_hid);
                        hid = true;
                    }
                    _ => {
                        debugln!("Unknown Descriptor Length {} Type {:X}", length as usize, descriptor_type);
                    }
                }
                i += length as isize;
            }

            memory::unalloc(desc_cfg_buf as usize);
        }
    }

    pub unsafe fn init(&mut self) {
        debugln!("UHCI on: {:X}, IRQ: {:X}", self.base, self.irq);

        let base = self.base as u16;
        let usbcmd = base;
        let usbsts = base + 02;
        let usbintr = base + 0x4;
        let frnum = base + 0x6;
        let flbaseadd = base + 0x8;
        let portsc1 = base + 0x10;
        let portsc2 = base + 0x12;

        debug::d(" CMD ");
        debug::dh(inw(usbcmd) as usize);
        outw(usbcmd, 1 << 2 | 1 << 1);
        debug::d(" to ");
        debug::dh(inw(usbcmd) as usize);

        outw(usbcmd, 0);
        debug::d(" to ");
        debug::dh(inw(usbcmd) as usize);

        debug::d(" STS ");
        debug::dh(inw(usbsts) as usize);

        debug::d(" INTR ");
        debug::dh(inw(usbintr) as usize);

        debug::d(" FRNUM ");
        debug::dh(inw(frnum) as usize);
        outw(frnum, 0);
        debug::d(" to ");
        debug::dh(inw(frnum) as usize);

        debug::d(" FLBASEADD ");
        debug::dh(ind(flbaseadd) as usize);
        for i in 0..1024 {
            self.frame_list.write(i, 1);
        }
        outd(flbaseadd, self.frame_list.address() as u32);
        debug::d(" to ");
        debug::dh(ind(flbaseadd) as usize);

        debug::d(" CMD ");
        debug::dh(inw(usbcmd) as usize);
        outw(usbcmd, 1);
        debug::d(" to ");
        debug::dh(inw(usbcmd) as usize);

        debug::dl();

        {
            debug::d(" PORTSC1 ");
            debug::dh(inw(portsc1) as usize);

            outw(portsc1, 1 << 9);
            debug::d(" to ");
            debug::dh(inw(portsc1) as usize);

            outw(portsc1, 0);
            debug::d(" to ");
            debug::dh(inw(portsc1) as usize);

            debug::dl();

            if inw(portsc1) & 1 == 1 {
                debug::d(" Device Found ");
                debug::dh(inw(portsc1) as usize);

                outw(portsc1, 4);
                debug::d(" to ");
                debug::dh(inw(portsc1) as usize);
                debug::dl();

                self.device(1);
            }
        }

        {
            debug::d(" PORTSC2 ");
            debug::dh(inw(portsc2) as usize);

            outw(portsc2, 1 << 9);
            debug::d(" to ");
            debug::dh(inw(portsc2) as usize);

            outw(portsc2, 0);
            debug::d(" to ");
            debug::dh(inw(portsc2) as usize);

            debug::dl();

            if inw(portsc2) & 1 == 1 {
                debug::d(" Device Found ");
                debug::dh(inw(portsc2) as usize);

                outw(portsc2, 4);
                debug::d(" to ");
                debug::dh(inw(portsc2) as usize);
                debug::dl();

                self.device(2);
            }
        }
    }
}
