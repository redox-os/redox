use alloc::boxed::Box;

use core::intrinsics::{volatile_load, volatile_store};
use core::{cmp, mem, ptr};

use common::context::{self, Context};
use common::debug;
use common::event::MouseEvent;
use common::memory::{self, Memory};
use common::scheduler;
use common::time::{self, Duration};

use drivers::pciconfig::PCIConfig;
use drivers::pio::*;

use schemes::KScheme;

pub struct UHCI {
    pub base: usize,
    pub irq: u8,
}

impl KScheme for UHCI {
    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            //d("UHCI IRQ\n");
        }
    }

    fn on_poll(&mut self) {
    }
}

#[repr(packed)]
struct SETUP {
    request_type: u8,
    request: u8,
    value: u16,
    index: u16,
    len: u16,
}

#[repr(packed)]
struct TD {
    link_ptr: u32,
    ctrl_sts: u32,
    token: u32,
    buffer: u32, // reserved: [u32; 4]
}

#[repr(packed)]
struct QH {
    head_ptr: u32,
    element_ptr: u32,
}

const DESC_DEV: u8 = 1;
#[repr(packed)]
struct DeviceDescriptor {
    length: u8,
    descriptor_type: u8,
    usb_version: u16,
    class: u8,
    sub_class: u8,
    protocol: u8,
    max_packet_size: u8,
    vendor: u16,
    product: u16,
    release: u16,
    manufacturer_string: u8,
    product_string: u8,
    serial_string: u8,
    configurations: u8,
}

impl DeviceDescriptor {
    fn new() -> Self {
        return DeviceDescriptor {
            length: 0,
            descriptor_type: 0,
            usb_version: 0,
            class: 0,
            sub_class: 0,
            protocol: 0,
            max_packet_size: 0,
            vendor: 0,
            product: 0,
            release: 0,
            manufacturer_string: 0,
            product_string: 0,
            serial_string: 0,
            configurations: 0,
        };
    }

    fn d(&self) {
        debug::d("Device Descriptor Length ");
        debug::dd(self.length as usize);
        debug::d(" Type ");
        debug::dh(self.descriptor_type as usize);
        debug::d(" USB ");
        debug::dh(self.usb_version as usize);
        debug::d(" Class ");
        debug::dh(self.class as usize);
        debug::d(" Subclass ");
        debug::dh(self.sub_class as usize);
        debug::d(" Protocol ");
        debug::dh(self.protocol as usize);
        debug::d(" Vendor ");
        debug::dh(self.vendor as usize);
        debug::d(" Product ");
        debug::dh(self.product as usize);
        debug::d(" Configurations ");
        debug::dh(self.configurations as usize);
        debug::dl();
    }
}

const DESC_CFG: u8 = 2;
#[repr(packed)]
struct ConfigDescriptor {
    length: u8,
    descriptor_type: u8,
    total_length: u16,
    interfaces: u8,
    number: u8,
    string: u8,
    attributes: u8,
    max_power: u8,
}

impl ConfigDescriptor {
    fn new() -> Self {
        return ConfigDescriptor {
            length: 0,
            descriptor_type: 0,
            total_length: 0,
            interfaces: 0,
            number: 0,
            string: 0,
            attributes: 0,
            max_power: 0,
        };
    }

    fn d(&self) {
        debug::d("Config Descriptor Length ");
        debug::dd(self.length as usize);
        debug::d(" Type ");
        debug::dh(self.descriptor_type as usize);
        debug::d(" Total Length ");
        debug::dd(self.total_length as usize);
        debug::d(" Interfaces ");
        debug::dd(self.interfaces as usize);
        debug::d(" Number ");
        debug::dd(self.number as usize);
        debug::d(" Attributes ");
        debug::dh(self.attributes as usize);
        debug::d(" Max Power ");
        debug::dd(self.max_power as usize);
        debug::dl();
    }
}

const DESC_INT: u8 = 4;
#[repr(packed)]
struct InterfaceDescriptor {
    length: u8,
    descriptor_type: u8,
    number: u8,
    alternate: u8,
    endpoints: u8,
    class: u8,
    sub_class: u8,
    protocol: u8,
    string: u8,
}

impl InterfaceDescriptor {
    fn new() -> Self {
        return InterfaceDescriptor {
            length: 0,
            descriptor_type: 0,
            number: 0,
            alternate: 0,
            endpoints: 0,
            class: 0,
            sub_class: 0,
            protocol: 0,
            string: 0,
        };
    }

    fn d(&self) {
        debug::d("Interface Descriptor Length ");
        debug::dd(self.length as usize);
        debug::d(" Type ");
        debug::dh(self.descriptor_type as usize);
        debug::d(" Number ");
        debug::dd(self.number as usize);
        debug::d(" Endpoints ");
        debug::dd(self.endpoints as usize);
        debug::d(" Class ");
        debug::dh(self.class as usize);
        debug::d(" Subclass ");
        debug::dh(self.sub_class as usize);
        debug::d(" Protocol ");
        debug::dh(self.protocol as usize);
        debug::dl();
    }
}

const DESC_END: u8 = 5;
#[repr(packed)]
struct EndpointDescriptor {
    length: u8,
    descriptor_type: u8,
    address: u8,
    attributes: u8,
    max_packet_size: u16,
    interval: u8,
}

impl EndpointDescriptor {
    fn new() -> Self {
        return EndpointDescriptor {
            length: 0,
            descriptor_type: 0,
            address: 0,
            attributes: 0,
            max_packet_size: 0,
            interval: 0,
        };
    }

    fn d(&self) {
        debug::d("Endpoint Descriptor Length ");
        debug::dd(self.length as usize);
        debug::d(" Type ");
        debug::dh(self.descriptor_type as usize);
        debug::d(" Address ");
        debug::dh(self.address as usize);
        debug::d(" Attributes ");
        debug::dh(self.attributes as usize);
        debug::d(" Interval ");
        debug::dh(self.interval as usize);
        debug::dl();
    }
}

const DESC_HID: u8 = 0x21;
#[repr(packed)]
struct HIDDescriptor {
    length: u8,
    descriptor_type: u8,
    hid_version: u16,
    country_code: u8,
    descriptors: u8,
    sub_descriptor_type: u8,
    sub_descriptor_length: u16,
}

impl HIDDescriptor {
    fn new() -> Self {
        return HIDDescriptor {
            length: 0,
            descriptor_type: 0,
            hid_version: 0,
            country_code: 0,
            descriptors: 0,
            sub_descriptor_type: 0,
            sub_descriptor_length: 0,
        };
    }

    fn d(&self) {
        debug::d("HID Descriptor Length ");
        debug::dd(self.length as usize);
        debug::d(" Type ");
        debug::dh(self.descriptor_type as usize);
        debug::d(" HID Version ");
        debug::dh(self.hid_version as usize);
        debug::d(" Country Code ");
        debug::dh(self.country_code as usize);
        debug::d(" Descriptors ");
        debug::dh(self.descriptors as usize);
        debug::d(" Sub Type ");
        debug::dh(self.sub_descriptor_type as usize);
        debug::d(" Sub Length ");
        debug::dd(self.sub_descriptor_length as usize);
        debug::dl();
    }
}

impl UHCI {
    pub unsafe fn new(mut pci: PCIConfig) -> Box<Self> {
        pci.flag(4, 4, true); // Bus mastering

        let module = box UHCI {
            base: pci.read(0x20) as usize & 0xFFFFFFF0,
            irq: pci.read(0x3C) as u8 & 0xF,
        };

        module.init();

        return module;
    }

    unsafe fn set_address(&self, frame_list: *mut u32, address: u8) {
        let base = self.base as u16;
        let frnum = PIO16::new(base + 6);

        let mut in_td = Memory::<TD>::new(1).unwrap();
        in_td.store(0,
                    TD {
                        link_ptr: 1,
                        ctrl_sts: 1 << 23,
                        token: 0x7FF << 21 | 0x69,
                        buffer: 0,
                    });

        let mut setup = Memory::<SETUP>::new(1).unwrap();
        setup.store(0,
                    SETUP {
                        request_type: 0b00000000,
                        request: 5,
                        value: address as u16,
                        index: 0,
                        len: 0,
                    });

        let mut setup_td = Memory::<TD>::new(1).unwrap();
        setup_td.store(0,
                       TD {
                           link_ptr: in_td.address() as u32 | 4,
                           ctrl_sts: 1 << 23,
                           token: (mem::size_of::<SETUP>() as u32 - 1) << 21 | 0x2D,
                           buffer: setup.address() as u32,
                       });

        let mut queue_head = Memory::<QH>::new(1).unwrap();
        queue_head.store(0,
                         QH {
                             head_ptr: 1,
                             element_ptr: setup_td.address() as u32,
                         });

        let frame = (frnum.read() + 2) & 0x3FF;
        ptr::write(frame_list.offset(frame as isize),
                   queue_head.address() as u32 | 2);

        loop {
            if setup_td.load(0).ctrl_sts & (1 << 23) == 0 {
                break;
            }

            let disable = scheduler::start_ints();
            Duration::new(0, 10 * time::NANOS_PER_MILLI).sleep();
            scheduler::end_ints(disable);
        }

        loop {
            if in_td.load(0).ctrl_sts & (1 << 23) == 0 {
                break;
            }

            let disable = scheduler::start_ints();
            Duration::new(0, 10 * time::NANOS_PER_MILLI).sleep();
            scheduler::end_ints(disable);
        }

        ptr::write(frame_list.offset(frame as isize), 1);
    }

    unsafe fn descriptor(&self,
                         frame_list: *mut u32,
                         address: u8,
                         descriptor_type: u8,
                         descriptor_index: u8,
                         descriptor_ptr: u32,
                         descriptor_len: u32) {
        let base = self.base as u16;
        let frnum = PIO16::new(base + 6);

        let mut out_td = Memory::<TD>::new(1).unwrap();
        out_td.store(0,
                     TD {
                         link_ptr: 1,
                         ctrl_sts: 1 << 23,
                         token: 0x7FF << 21 | (address as u32) << 8 | 0xE1,
                         buffer: 0,
                     });

        let mut in_td = Memory::<TD>::new(1).unwrap();
        in_td.store(0,
                    TD {
                        link_ptr: out_td.address() as u32 | 4,
                        ctrl_sts: 1 << 23,
                        token: (descriptor_len - 1) << 21 | (address as u32) << 8 | 0x69,
                        buffer: descriptor_ptr,
                    });

        let mut setup = Memory::<SETUP>::new(1).unwrap();
        setup.store(0,
                    SETUP {
                        request_type: 0b10000000,
                        request: 6,
                        value: (descriptor_type as u16) << 8 | (descriptor_index as u16),
                        index: 0,
                        len: descriptor_len as u16,
                    });

        let mut setup_td = Memory::<TD>::new(1).unwrap();
        setup_td.store(0,
                       TD {
                           link_ptr: in_td.address() as u32 | 4,
                           ctrl_sts: 1 << 23,
                           token: (mem::size_of::<SETUP>() as u32 - 1) << 21 |
                                  (address as u32) << 8 | 0x2D,
                           buffer: setup.address() as u32,
                       });

        let mut queue_head = Memory::<QH>::new(1).unwrap();
        queue_head.store(0,
                         QH {
                             head_ptr: 1,
                             element_ptr: setup_td.address() as u32,
                         });

        let frame = (frnum.read() + 2) & 0x3FF;
        ptr::write(frame_list.offset(frame as isize),
                   queue_head.address() as u32 | 2);

        loop {
            if setup_td.load(0).ctrl_sts & (1 << 23) == 0 {
                break;
            }

            let disable = scheduler::start_ints();
            Duration::new(0, 10 * time::NANOS_PER_MILLI).sleep();
            scheduler::end_ints(disable);
        }

        loop {
            if in_td.load(0).ctrl_sts & (1 << 23) == 0 {
                break;
            }

            let disable = scheduler::start_ints();
            Duration::new(0, 10 * time::NANOS_PER_MILLI).sleep();
            scheduler::end_ints(disable);
        }

        loop {
            if out_td[0].ctrl_sts & (1 << 23) == 0 {
                break;
            }

            let disable = scheduler::start_ints();
            Duration::new(0, 10 * time::NANOS_PER_MILLI).sleep();
            scheduler::end_ints(disable);
        }

        ptr::write(frame_list.offset(frame as isize), 1);
    }

    unsafe fn device(&self, frame_list: *mut u32, address: u8) {
        self.set_address(frame_list, address);

        let desc_dev: *mut DeviceDescriptor = memory::alloc_type();
        ptr::write(desc_dev, DeviceDescriptor::new());
        self.descriptor(frame_list,
                        address,
                        DESC_DEV,
                        0,
                        desc_dev as u32,
                        mem::size_of_val(&*desc_dev) as u32);
        //(*desc_dev).d();

        for configuration in 0..(*desc_dev).configurations {
            let desc_cfg_len = 1023;
            let desc_cfg_buf = memory::alloc(desc_cfg_len) as *mut u8;
            for i in 0..desc_cfg_len as isize {
                ptr::write(desc_cfg_buf.offset(i), 0);
            }
            self.descriptor(frame_list,
                            address,
                            DESC_CFG,
                            configuration,
                            desc_cfg_buf as u32,
                            desc_cfg_len as u32);

            let desc_cfg = ptr::read(desc_cfg_buf as *const ConfigDescriptor);
            //desc_cfg.d();

            let mut i = desc_cfg.length as isize;
            while i < desc_cfg.total_length as isize {
                let length = ptr::read(desc_cfg_buf.offset(i));
                let descriptor_type = ptr::read(desc_cfg_buf.offset(i + 1));
                match descriptor_type {
                    DESC_INT => {
                        //let desc_int = ptr::read(desc_cfg_buf.offset(i) as *const InterfaceDescriptor);
                        //desc_int.d();
                    }
                    DESC_END => {
                        let desc_end = ptr::read(desc_cfg_buf.offset(i) as *const EndpointDescriptor);
                        //desc_end.d();

                        let endpoint = desc_end.address & 0xF;
                        let in_len = desc_end.max_packet_size as usize;

                        let base = self.base as u16;
                        let frnum = base + 0x6;

                        Context::spawn(box move || {
                            let in_ptr = memory::alloc(in_len) as *mut u8;
                            let in_td: *mut TD = memory::alloc_type();

                            loop {
                                for i in 0..in_len as isize {
                                    volatile_store(in_ptr.offset(i), 0);
                                }

                                ptr::write(in_td,
                                           TD {
                                               link_ptr: 1,
                                               ctrl_sts: 1 << 25 | 1 << 23,
                                               token: (in_len as u32 - 1) << 21 |
                                                      (endpoint as u32) << 15 |
                                                      (address as u32) << 8 |
                                                      0x69,
                                               buffer: in_ptr as u32,
                                           });

                                let reenable = scheduler::start_no_ints();
                                let frame = (inw(frnum) + 2) & 0x3FF;
                                volatile_store(frame_list.offset(frame as isize), in_td as u32);
                                scheduler::end_no_ints(reenable);

                                loop {
                                    let ctrl_sts = volatile_load(in_td).ctrl_sts;
                                    if ctrl_sts & (1 << 23) == 0 {
                                        break;
                                    }

                                    context::context_switch(false);
                                }

                                volatile_store(frame_list.offset(frame as isize), 1);

                                if volatile_load(in_td).ctrl_sts & 0x7FF > 0 {
                                    let buttons = ptr::read(in_ptr.offset(0) as *const u8) as usize;
                                    let x = ptr::read(in_ptr.offset(1) as *const u16) as usize;
                                    let y = ptr::read(in_ptr.offset(3) as *const u16) as usize;

                                    //TODO: will the session need to see this info still?
                                    // LazyOxen
                                    /*
                                    let mouse_x = (x * (*::session_ptr).display.width) / 32768;
                                    let mouse_y = (y * (*::session_ptr).display.height) / 32768;

                                    (*::session_ptr).mouse_point.x =
                                        cmp::max(0,
                                                 cmp::min((*::session_ptr).display.width as isize -
                                                          1,
                                                          mouse_x as isize));
                                    (*::session_ptr).mouse_point.y =
                                        cmp::max(0,
                                                 cmp::min((*::session_ptr).display.height as isize -
                                                          1,
                                                          mouse_y as isize));
                                    */

                                    MouseEvent {
                                        x: 0, 
                                        y: y as isize,
                                        left_button: buttons & 1 == 1,
                                        middle_button: buttons & 4 == 4,
                                        right_button: buttons & 2 == 2,
                                    }
                                        .trigger();
                                }

                                Duration::new(0, 10 * time::NANOS_PER_MILLI).sleep();
                            }

                            //memory::unalloc(in_td as usize);
                        });
                    }
                    DESC_HID => {
                        let desc_hid = &*(desc_cfg_buf.offset(i) as *const HIDDescriptor);
                        desc_hid.d();
                    }
                    _ => {
                        debug::d("Unknown Descriptor Length ");
                        debug::dd(length as usize);
                        debug::d(" Type ");
                        debug::dh(descriptor_type as usize);
                        debug::dl();
                    }
                }
                i += length as isize;
            }

            memory::unalloc(desc_cfg_buf as usize);
        }

        memory::unalloc(desc_dev as usize);
    }

    pub unsafe fn init(&self) {
        debug::d("UHCI on: ");
        debug::dh(self.base);
        debug::d(", IRQ: ");
        debug::dbh(self.irq);

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
        let disable = scheduler::start_ints();
        Duration::new(0, 100 * time::NANOS_PER_MILLI).sleep();
        scheduler::end_ints(disable);
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
        let frame_list = memory::alloc(1024 * 4) as *mut u32;
        for i in 0..1024 {
            ptr::write(frame_list.offset(i), 1);
        }
        outd(flbaseadd, frame_list as u32);
        debug::d(" to ");
        debug::dh(ind(flbaseadd) as usize);

        debug::d(" CMD ");
        debug::dh(inw(usbcmd) as usize);
        outw(usbcmd, 1);
        debug::d(" to ");
        debug::dh(inw(usbcmd) as usize);

        debug::dl();

        let disable = scheduler::start_ints();
        Duration::new(0, 100 * time::NANOS_PER_MILLI).sleep();
        scheduler::end_ints(disable);

        {
            debug::d(" PORTSC1 ");
            debug::dh(inw(portsc1) as usize);

            outw(portsc1, 1 << 9);
            debug::d(" to ");
            debug::dh(inw(portsc1) as usize);

            let disable = scheduler::start_ints();
            Duration::new(0, 100 * time::NANOS_PER_MILLI).sleep();
            scheduler::end_ints(disable);

            outw(portsc1, 0);
            debug::d(" to ");
            debug::dh(inw(portsc1) as usize);

            let disable = scheduler::start_ints();
            Duration::new(0, 100 * time::NANOS_PER_MILLI).sleep();
            scheduler::end_ints(disable);

            debug::dl();

            if inw(portsc1) & 1 == 1 {
                debug::d(" Device Found ");
                debug::dh(inw(portsc1) as usize);

                outw(portsc1, 4);
                debug::d(" to ");
                debug::dh(inw(portsc1) as usize);
                debug::dl();

                let disable = scheduler::start_ints();
                Duration::new(0, 100 * time::NANOS_PER_MILLI).sleep();
                scheduler::end_ints(disable);

                self.device(frame_list, 1);
            }
        }

        {
            debug::d(" PORTSC2 ");
            debug::dh(inw(portsc2) as usize);

            outw(portsc2, 1 << 9);
            debug::d(" to ");
            debug::dh(inw(portsc2) as usize);

            let disable = scheduler::start_ints();
            Duration::new(0, 100 * time::NANOS_PER_MILLI).sleep();
            scheduler::end_ints(disable);

            outw(portsc2, 0);
            debug::d(" to ");
            debug::dh(inw(portsc2) as usize);

            let disable = scheduler::start_ints();
            Duration::new(0, 100 * time::NANOS_PER_MILLI).sleep();
            scheduler::end_ints(disable);

            debug::dl();

            if inw(portsc2) & 1 == 1 {
                debug::d(" Device Found ");
                debug::dh(inw(portsc2) as usize);

                outw(portsc2, 4);
                debug::d(" to ");
                debug::dh(inw(portsc2) as usize);
                debug::dl();

                let disable = scheduler::start_ints();
                Duration::new(0, 100 * time::NANOS_PER_MILLI).sleep();
                scheduler::end_ints(disable);

                self.device(frame_list, 2);
            }
        }
    }
}
