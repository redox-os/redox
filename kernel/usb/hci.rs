use arch::context::{context_switch, Context};
use arch::memory;

use common::event::MouseEvent;
use common::time::{self, Duration};

use core::{cmp, mem, ptr, slice};

use graphics::display::VBEMODEINFO;

use super::{Packet, Pipe, Setup};
use super::desc::*;

pub trait Hci {
    fn msg(&mut self, address: u8, endpoint: u8, pipe: Pipe, msgs: &[Packet]) -> usize;

    fn descriptor(&mut self,
                         address: u8,
                         descriptor_type: u8,
                         descriptor_index: u8,
                         descriptor_ptr: usize,
                         descriptor_len: usize) {
        self.msg(address, 0, Pipe::Control, &[
            Packet::Setup(&Setup::get_descriptor(descriptor_type, descriptor_index, 0, descriptor_len as u16)),
            Packet::In(&mut unsafe { slice::from_raw_parts_mut(descriptor_ptr as *mut u8, descriptor_len as usize) }),
            Packet::Out(&[])
        ]);
    }

    unsafe fn device(&mut self, address: u8) where Self: Sized + 'static {
        self.msg(0, 0, Pipe::Control, &[
            Packet::Setup(&Setup::set_address(address)),
            Packet::In(&mut [])
        ]);

        let mut desc_dev = box DeviceDescriptor::default();
        self.descriptor(address,
                        DESC_DEV,
                        0,
                        (&mut *desc_dev as *mut DeviceDescriptor) as usize,
                        mem::size_of_val(&*desc_dev));
        debugln!("{:#?}", *desc_dev);

        if desc_dev.manufacturer_string > 0 {
            let mut desc_str = box StringDescriptor::default();
            self.descriptor(address,
                            DESC_STR,
                            desc_dev.manufacturer_string,
                            (&mut *desc_str as *mut StringDescriptor) as usize,
                            mem::size_of_val(&*desc_str));
            debugln!("Manufacturer: {}", desc_str.str());
        }

        if desc_dev.product_string > 0 {
            let mut desc_str = box StringDescriptor::default();
            self.descriptor(address,
                            DESC_STR,
                            desc_dev.product_string,
                            (&mut *desc_str as *mut StringDescriptor) as usize,
                            mem::size_of_val(&*desc_str));
            debugln!("Product: {}", desc_str.str());
        }

        if desc_dev.serial_string > 0 {
            let mut desc_str = box StringDescriptor::default();
            self.descriptor(address,
                            DESC_STR,
                            desc_dev.serial_string,
                            (&mut *desc_str as *mut StringDescriptor) as usize,
                            mem::size_of_val(&*desc_str));
            debugln!("Serial: {}", desc_str.str());
        }

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

            if desc_cfg.string > 0 {
                let mut desc_str = box StringDescriptor::default();
                self.descriptor(address,
                                DESC_STR,
                                desc_cfg.string,
                                (&mut *desc_str as *mut StringDescriptor) as usize,
                                mem::size_of_val(&*desc_str));
                debugln!("Configuration: {}", desc_str.str());
            }

            let mut hid = false;

            let mut i = desc_cfg.length as isize;
            while i < desc_cfg.total_length as isize {
                let length = ptr::read(desc_cfg_buf.offset(i));
                let descriptor_type = ptr::read(desc_cfg_buf.offset(i + 1));
                match descriptor_type {
                    DESC_INT => {
                        let desc_int = ptr::read(desc_cfg_buf.offset(i) as *const InterfaceDescriptor);
                        debugln!("{:#?}", desc_int);

                        if desc_int.string > 0 {
                            let mut desc_str = box StringDescriptor::default();
                            self.descriptor(address,
                                            DESC_STR,
                                            desc_int.string,
                                            (&mut *desc_str as *mut StringDescriptor) as usize,
                                            mem::size_of_val(&*desc_str));
                            debugln!("Interface: {}", desc_str.str());
                        }
                    }
                    DESC_END => {
                        let desc_end = ptr::read(desc_cfg_buf.offset(i) as *const EndpointDescriptor);
                        debugln!("{:#?}", desc_end);

                        let endpoint = desc_end.address & 0xF;
                        let in_len = desc_end.max_packet_size as usize;

                        if hid {
                            let this = self as *mut Hci;
                            Context::spawn("kuhci_hid".into(),
                                           box move || {
                                if let Some(mode_info) = VBEMODEINFO {
                                    debugln!("Starting HID driver");

                                    let in_ptr = memory::alloc_aligned(in_len, 4096) as *mut u8;

                                    loop {
                                        for i in 0..in_len as isize {
                                            ptr::write(in_ptr.offset(i), 0);
                                        }

                                        if (*this).msg(address, endpoint, Pipe::Isochronous, &[
                                            Packet::In(&mut slice::from_raw_parts_mut(in_ptr, in_len))
                                        ]) > 0 {
                                            let buttons = ptr::read(in_ptr.offset(0) as *const u8) as usize;
                                            let x = ptr::read(in_ptr.offset(1) as *const u16) as usize;
                                            let y = ptr::read(in_ptr.offset(3) as *const u16) as usize;

                                            let mouse_x = (x * mode_info.xresolution as usize) / 32768;
                                            let mouse_y = (y * mode_info.yresolution as usize) / 32768;

                                            let mouse_event = MouseEvent {
                                                x: cmp::max(0, cmp::min(mode_info.xresolution as i32 - 1, mouse_x as i32)),
                                                y: cmp::max(0, cmp::min(mode_info.yresolution as i32 - 1, mouse_y as i32)),
                                                left_button: buttons & 1 == 1,
                                                middle_button: buttons & 4 == 4,
                                                right_button: buttons & 2 == 2,
                                            };

                                            if ::env().console.lock().draw {
                                                //ignore mouse event
                                            } else {
                                                ::env().events.send(mouse_event.to_event());
                                            }
                                        }

                                        {
                                            let mut contexts = ::env().contexts.lock();
                                            if let Ok(mut current) = contexts.current_mut() {
                                                current.blocked = true;
                                                current.wake = Some(Duration::monotonic() + Duration::new(0, 10 * time::NANOS_PER_MILLI));
                                            }
                                        }

                                        context_switch();
                                    }

                                    //memory::unalloc(in_ptr as usize);
                                }
                            });
                        }
                    }
                    DESC_HID => {
                        //let desc_hid = &*(desc_cfg_buf.offset(i) as *const HIDDescriptor);
                        //debugln!("{:#?}", desc_hid);
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
}
