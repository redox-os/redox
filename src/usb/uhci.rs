use core::intrinsics::{volatile_load, volatile_store};
use core::ptr::{read, write};

use common::context::*;
use common::memory::*;
use common::pci::*;
use common::pio::*;
use common::scheduler::*;

use programs::common::*;

pub struct UHCI {
    pub base: usize,
    pub irq: u8
}

impl SessionItem for UHCI {
    fn on_irq(&mut self, irq: u8){
        if irq == self.irq {
            d("UHCI IRQ\n");
        }
    }

    fn on_poll(&mut self){
    }
}

#[repr(packed)]
struct SETUP {
    request_type: u8,
    request: u8,
    value: u16,
    index: u16,
    len: u16
}

#[repr(packed)]
struct TD {
    link_ptr: u32,
    ctrl_sts: u32,
    token: u32,
    buffer: u32
    //reserved: [u32; 4]
}

#[repr(packed)]
struct QH {
    head_ptr: u32,
    element_ptr: u32
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
    configurations: u8
}

impl DeviceDescriptor {
    fn new() -> DeviceDescriptor {
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
            configurations: 0
        };
    }

    fn d(&self){
        d("Device Descriptor Length ");
        dd(self.length as usize);
        d(" Type ");
        dh(self.descriptor_type as usize);
        d(" USB ");
        dh(self.usb_version as usize);
        d(" Class ");
        dh(self.class as usize);
        d(" Subclass ");
        dh(self.sub_class as usize);
        d(" Protocol ");
        dh(self.protocol as usize);
        d(" Vendor ");
        dh(self.vendor as usize);
        d(" Product ");
        dh(self.product as usize);
        d(" Configurations ");
        dh(self.configurations as usize);
        dl();
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
    max_power: u8
}

impl ConfigDescriptor {
    fn new() -> ConfigDescriptor {
        return ConfigDescriptor {
            length: 0,
            descriptor_type: 0,
            total_length: 0,
            interfaces: 0,
            number: 0,
            string: 0,
            attributes: 0,
            max_power: 0
        };
    }

    fn d(&self){
        d("Config Descriptor Length ");
        dd(self.length as usize);
        d(" Type ");
        dh(self.descriptor_type as usize);
        d(" Total Length ");
        dd(self.total_length as usize);
        d(" Interfaces ");
        dd(self.interfaces as usize);
        d(" Number ");
        dd(self.number as usize);
        d(" Attributes ");
        dh(self.attributes as usize);
        d(" Max Power ");
        dd(self.max_power as usize);
        dl();
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
    string: u8
}

impl InterfaceDescriptor {
    fn new() -> InterfaceDescriptor {
        return InterfaceDescriptor {
            length: 0,
            descriptor_type: 0,
            number: 0,
            alternate: 0,
            endpoints: 0,
            class: 0,
            sub_class: 0,
            protocol: 0,
            string: 0
        };
    }

    fn d(&self){
        d("Interface Descriptor Length ");
        dd(self.length as usize);
        d(" Type ");
        dh(self.descriptor_type as usize);
        d(" Number ");
        dd(self.number as usize);
        d(" Endpoints ");
        dd(self.endpoints as usize);
        d(" Class ");
        dh(self.class as usize);
        d(" Subclass ");
        dh(self.sub_class as usize);
        d(" Protocol ");
        dh(self.protocol as usize);
        dl();
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
    interval: u8
}

impl EndpointDescriptor {
    fn new() -> EndpointDescriptor {
        return EndpointDescriptor {
            length: 0,
            descriptor_type: 0,
            address: 0,
            attributes: 0,
            max_packet_size: 0,
            interval: 0
        };
    }

    fn d(&self){
        d("Endpoint Descriptor Length ");
        dd(self.length as usize);
        d(" Type ");
        dh(self.descriptor_type as usize);
        d(" Address ");
        dh(self.address as usize);
        d(" Attributes ");
        dh(self.attributes as usize);
        d(" Interval ");
        dh(self.interval as usize);
        dl();
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
    sub_descriptor_length: u16
}

impl HIDDescriptor {
    fn new() -> HIDDescriptor {
        return HIDDescriptor {
            length: 0,
            descriptor_type: 0,
            hid_version: 0,
            country_code: 0,
            descriptors: 0,
            sub_descriptor_type: 0,
            sub_descriptor_length: 0
        };
    }

    fn d(&self){
        d("HID Descriptor Length ");
        dd(self.length as usize);
        d(" Type ");
        dh(self.descriptor_type as usize);
        d(" HID Version ");
        dh(self.hid_version as usize);
        d(" Country Code ");
        dh(self.country_code as usize);
        d(" Descriptors ");
        dh(self.descriptors as usize);
        d(" Sub Type ");
        dh(self.sub_descriptor_type as usize);
        d(" Sub Length ");
        dd(self.sub_descriptor_length as usize);
        dl();
    }
}

impl UHCI {
    pub unsafe fn new(bus: usize, slot: usize, func: usize) -> Box<UHCI> {
        pci_write(bus, slot, func, 0x04, pci_read(bus, slot, func, 4) | 4); // Bus mastering

        let mut module = box UHCI {
            base: pci_read(bus, slot, func, 0x20) & 0xFFFFFFF0,
            irq: pci_read(bus, slot, func, 0x3C) as u8 & 0xF
        };

        module.init();

        return module;
    }

    unsafe fn set_address(&self, frame_list: *mut u32, address: u8){
        let base = self.base as u16;
        let usbcmd = base;
        let usbsts = base + 2;
        let usbintr = base + 4;
        let frnum = base + 6;

        let in_td: *mut TD = alloc_type();
        ptr::write(in_td, TD {
            link_ptr: 1,
            ctrl_sts: 1 << 23,
            token: 0x7FF << 21 | 0x69,
            buffer: 0
        });

        let setup: *mut SETUP = alloc_type();
        write(setup, SETUP {
            request_type: 0b00000000,
            request: 5,
            value: address as u16,
            index: 0,
            len: 0
        });

        let setup_td: *mut TD = alloc_type();
        write(setup_td, TD {
            link_ptr: in_td as u32 | 4,
            ctrl_sts: 1 << 23,
            token: (size_of_val(&*setup) as u32 - 1) << 21 | 0x2D,
            buffer: setup as u32
        });

        let queue_head: *mut QH = alloc_type();
        write(queue_head, QH {
            head_ptr: 1,
            element_ptr: setup_td as u32
        });

        let frame = (inw(frnum) + 2) & 0x3FF;
        write(frame_list.offset(frame as isize), queue_head as u32 | 2);

        loop {
            if (*setup_td).ctrl_sts & (1 << 23) == 0 {
                d("SETUP_TD ");
                dh((*setup_td).ctrl_sts as usize);
                dl();
                break;
            }

            let disable = start_ints();
            Duration::new(0, 10*NANOS_PER_MILLI).sleep();
            end_ints(disable);
        }

        loop {
            if (*in_td).ctrl_sts & (1 << 23) == 0 {
                d("IN_TD ");
                dh((*in_td).ctrl_sts as usize);
                dl();
                break;
            }

            let disable = start_ints();
            Duration::new(0, 10*NANOS_PER_MILLI).sleep();
            end_ints(disable);
        }

        write(frame_list.offset(frame as isize), 1);

        unalloc(queue_head as usize);
        unalloc(setup_td as usize);
        unalloc(setup as usize);
        unalloc(in_td as usize);
    }

    unsafe fn descriptor(&self, frame_list: *mut u32, address: u8, descriptor_type: u8, descriptor_index: u8, descriptor_ptr: u32, descriptor_len: u32){
        let base = self.base as u16;
        let usbcmd = base;
        let frnum = base + 6;

        let out_td: *mut TD = alloc_type();
        ptr::write(out_td, TD {
            link_ptr: 1,
            ctrl_sts: 1 << 23,
            token: 0x7FF << 21 | (address as u32) << 8 |  0xE1,
            buffer: 0
        });

        let in_td: *mut TD = alloc_type();
        ptr::write(in_td, TD {
            link_ptr: out_td as u32 | 4,
            ctrl_sts: 1 << 23,
            token: (descriptor_len - 1) << 21 | (address as u32) << 8 | 0x69,
            buffer: descriptor_ptr
        });

        let setup: *mut SETUP = alloc_type();
        write(setup, SETUP {
            request_type: 0b10000000,
            request: 6,
            value: (descriptor_type as u16) << 8 | (descriptor_index as u16),
            index: 0,
            len: descriptor_len as u16
        });

        let setup_td: *mut TD = alloc_type();
        write(setup_td, TD {
            link_ptr: in_td as u32 | 4,
            ctrl_sts: 1 << 23,
            token: (size_of_val(&*setup) as u32 - 1) << 21 | (address as u32) << 8 | 0x2D,
            buffer: setup as u32
        });

        let queue_head: *mut QH = alloc_type();
        write(queue_head, QH {
            head_ptr: 1,
            element_ptr: setup_td as u32
        });

        let frame = (inw(frnum) + 2) & 0x3FF;
        write(frame_list.offset(frame as isize), queue_head as u32 | 2);

        loop {
            if (*setup_td).ctrl_sts & (1 << 23) == 0 {
                d("SETUP_TD ");
                dh((*setup_td).ctrl_sts as usize);
                dl();
                break;
            }

            let disable = start_ints();
            Duration::new(0, 10*NANOS_PER_MILLI).sleep();
            end_ints(disable);
        }

        loop {
            if (*in_td).ctrl_sts & (1 << 23) == 0 {
                d("IN_TD ");
                dh((*in_td).ctrl_sts as usize);
                dl();
                break;
            }

            let disable = start_ints();
            Duration::new(0, 10*NANOS_PER_MILLI).sleep();
            end_ints(disable);
        }

        loop {
            if (*out_td).ctrl_sts & (1 << 23) == 0 {
                d("OUT_TD ");
                dh((*out_td).ctrl_sts as usize);
                dl();
                break;
            }

            let disable = start_ints();
            Duration::new(0, 10*NANOS_PER_MILLI).sleep();
            end_ints(disable);
        }

        write(frame_list.offset(frame as isize), 1);

        unalloc(queue_head as usize);
        unalloc(setup_td as usize);
        unalloc(setup as usize);
        unalloc(in_td as usize);
        unalloc(out_td as usize);
    }

    unsafe fn device(&self, frame_list: *mut u32, address: u8){
        self.set_address(frame_list, address);

        let desc_dev: *mut DeviceDescriptor = alloc_type();
        write(desc_dev, DeviceDescriptor::new());
        self.descriptor(frame_list, address, DESC_DEV, 0, desc_dev as u32, size_of_val(&*desc_dev) as u32);
        (*desc_dev).d();

        for configuration in 0..(*desc_dev).configurations {
            let desc_cfg_len = 1023;
            let desc_cfg_buf = alloc(desc_cfg_len) as *mut u8;
            for i in 0..desc_cfg_len as isize {
                write(desc_cfg_buf.offset(i), 0);
            }
            self.descriptor(frame_list, address, DESC_CFG, configuration, desc_cfg_buf as u32, desc_cfg_len as u32);

            let desc_cfg = read(desc_cfg_buf as *const ConfigDescriptor);
            desc_cfg.d();

            let mut i = desc_cfg.length as isize;
            while i < desc_cfg.total_length as isize {
                let length = read(desc_cfg_buf.offset(i));
                let descriptor_type = read(desc_cfg_buf.offset(i + 1));
                match descriptor_type {
                    DESC_INT => {
                        let desc_int = read(desc_cfg_buf.offset(i) as *const InterfaceDescriptor);
                        desc_int.d();
                    },
                    DESC_END => {
                        let desc_end = read(desc_cfg_buf.offset(i) as *const EndpointDescriptor);
                        desc_end.d();

                        let endpoint = desc_end.address & 0xF;
                        let in_len = desc_end.max_packet_size as usize;

                        let base = self.base as u16;
                        let usbcmd = base;
                        let frnum = base + 0x6;

                        Context::spawn(box move ||{
                            let in_ptr = alloc(in_len) as *mut u8;
                            let in_td: *mut TD = alloc_type();

                            loop {
                                for i in 0..in_len as isize {
                                    volatile_store(in_ptr.offset(i), 0);
                                }

                                write(in_td, TD {
                                    link_ptr: 1,
                                    ctrl_sts: 1 << 25 | 1 << 23,
                                    token: (in_len as u32 - 1) << 21 | (endpoint as u32) << 15 | (address as u32) << 8 | 0x69,
                                    buffer: in_ptr as u32
                                });

                                let reenable = start_no_ints();
                                    let frame = (inw(frnum) + 2) & 0x3FF;
                                    volatile_store(frame_list.offset(frame as isize), in_td as u32);
                                end_no_ints(reenable);

                                loop {
                                    let ctrl_sts = volatile_load(in_td).ctrl_sts;
                                    if ctrl_sts & (1 << 23) == 0 {
                                        break;
                                    }

                                    sys_yield();
                                }

                                volatile_store(frame_list.offset(frame as isize), 1);

                                if volatile_load(in_td).ctrl_sts & 0x7FF > 0 {
                                    let buttons = read(in_ptr.offset(0) as *const u8) as usize;
                                    let x = read(in_ptr.offset(1) as *const u16) as usize;
                                    let y = read(in_ptr.offset(3) as *const u16) as usize;

                                    let mouse_x = (x * (*::session_ptr).display.width)/32768;
                                    let mouse_y = (y * (*::session_ptr).display.height)/32768;

                                    (*::session_ptr).mouse_point.x = max(0, min((*::session_ptr).display.width as isize - 1, mouse_x as isize));
                                    (*::session_ptr).mouse_point.y = max(0, min((*::session_ptr).display.height as isize - 1, mouse_y as isize));

                                    MouseEvent {
                                        x: 0,
                                        y: 0,
                                        left_button: buttons & 1 == 1,
                                        middle_button: buttons & 4 == 4,
                                        right_button: buttons & 2 == 2,
                                        valid: true
                                    }.trigger();
                                }

                                Duration::new(0, 10*NANOS_PER_MILLI).sleep();
                            }

                            unalloc(in_td as usize);
                        });
                    },
                    DESC_HID => {
                        let desc_hid = &*(desc_cfg_buf.offset(i) as *const HIDDescriptor);
                        desc_hid.d();
                    },
                    _ => {
                        d("Unknown Descriptor Length ");
                        dd(length as usize);
                        d(" Type ");
                        dh(descriptor_type as usize);
                        dl();
                    }
                }
                i += length as isize;
            }

            unalloc(desc_cfg_buf as usize);
        }

        unalloc(desc_dev as usize);
    }

    pub unsafe fn init(&self){
        d("UHCI on: ");
        dh(self.base);
        d(", IRQ: ");
        dbh(self.irq);

        let base = self.base as u16;
        let usbcmd = base;
        let usbsts = base + 02;
        let usbintr = base + 0x4;
        let frnum = base + 0x6;
        let flbaseadd = base + 0x8;
        let sofmod = base + 0xC;
        let portsc1 = base + 0x10;
        let portsc2 = base + 0x12;

        d(" CMD ");
        dh(inw(usbcmd) as usize);
            outw(usbcmd, 1 << 2 | 1 << 1);
        d(" to ");
        dh(inw(usbcmd) as usize);
            let disable = start_ints();
            Duration::new(0, 10*NANOS_PER_MILLI).sleep();
            end_ints(disable);
        outw(usbcmd, 0);
        d(" to ");
        dh(inw(usbcmd) as usize);

        d(" STS ");
        dh(inw(usbsts) as usize);

        d(" INTR ");
        dh(inw(usbintr) as usize);

        d(" FRNUM ");
        dh(inw(frnum) as usize);
            outw(frnum, 0);
        d(" to ");
        dh(inw(frnum) as usize);

        d(" FLBASEADD ");
        dh(ind(flbaseadd) as usize);
            let frame_list = alloc(1024 * 4) as *mut u32;
            for i in 0..1024 {
                write(frame_list.offset(i), 1);
            }
            outd(flbaseadd, frame_list as u32);
        d(" to ");
        dh(ind(flbaseadd) as usize);

        d(" CMD ");
        dh(inw(usbcmd) as usize);
            outw(usbcmd, 1);
        d(" to ");
        dh(inw(usbcmd) as usize);

        dl();

        d(" PORTSC1 ");
        dh(inw(portsc1) as usize);
        dl();

        if inw(portsc1) & 1 == 1 {
            outw(portsc1, 1 << 9);
            d("Device Found ");
            dh(inw(portsc1) as usize);

            let disable = start_ints();
            Duration::new(0, 10*NANOS_PER_MILLI).sleep();
            end_ints(disable);

            outw(portsc1, 0);
            d(" to ");
            dh(inw(portsc1) as usize);

            let disable = start_ints();
            Duration::new(0, 10*NANOS_PER_MILLI).sleep();
            end_ints(disable);

            outw(portsc1, 4);
            d(" to ");
            dh(inw(portsc1) as usize);
            dl();

            self.device(frame_list, 1);
        }

        d(" PORTSC2 ");
        dh(inw(portsc2) as usize);
        dl();

        if inw(portsc2) & 1 == 1 {
            outw(portsc2, 1 << 9);
            d("Device Found ");
            dh(inw(portsc2) as usize);

            let disable = start_ints();
            Duration::new(0, 10*NANOS_PER_MILLI).sleep();
            end_ints(disable);

            outw(portsc2, 0);
            d(" to ");
            dh(inw(portsc2) as usize);

            let disable = start_ints();
            Duration::new(0, 10*NANOS_PER_MILLI).sleep();
            end_ints(disable);

            outw(portsc2, 4);
            d(" to ");
            dh(inw(portsc2) as usize);
            dl();

            self.device(frame_list, 2);
        }
    }
}
