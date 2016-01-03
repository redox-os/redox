use collections::string::ToString;

use common::event::MouseEvent;
use common::memory;
use common::time::{self, Duration};

use core::{cmp, mem, ptr, slice};

use graphics::display::VBEMODEINFO;

use scheduler::Context;

use super::desc::*;
use super::setup::Setup;

#[derive(Debug)]
pub enum UsbMsg<'a> {
    Setup(&'a Setup),
    In(&'a mut [u8]),
    InIso(&'a mut [u8]),
    Out(&'a [u8]),
    OutIso(&'a [u8]),
}

pub trait UsbHci {
    fn msg(&mut self, address: u8, endpoint: u8, msgs: &[UsbMsg]) -> usize;

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

    unsafe fn device(&mut self, address: u8) where Self: Sized + 'static {
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
                            let this = self as *mut UsbHci;
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
}
