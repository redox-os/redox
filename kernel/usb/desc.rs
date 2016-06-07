use core::{cmp, str};

pub const DESC_DEV: u8 = 1;
#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
pub struct DeviceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub usb_version: u16,
    pub class: u8,
    pub sub_class: u8,
    pub protocol: u8,
    pub max_packet_size: u8,
    pub vendor: u16,
    pub product: u16,
    pub release: u16,
    pub manufacturer_string: u8,
    pub product_string: u8,
    pub serial_string: u8,
    pub configurations: u8,
}

pub const DESC_CFG: u8 = 2;
#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
pub struct ConfigDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub total_length: u16,
    pub interfaces: u8,
    pub number: u8,
    pub string: u8,
    pub attributes: u8,
    pub max_power: u8,
}

pub const DESC_STR: u8 = 3;
#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
pub struct StringDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    // TODO: Support longer strings
    pub data: [u8; 32],
}

impl StringDescriptor {
    pub fn str<'a>(&'a self) -> &'a str {
        unsafe {
            str::from_utf8_unchecked(&self.data[..cmp::min(self.length as usize, self.data.len())])
        }
    }
}

pub const DESC_INT: u8 = 4;
#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
pub struct InterfaceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub number: u8,
    pub alternate: u8,
    pub endpoints: u8,
    pub class: u8,
    pub sub_class: u8,
    pub protocol: u8,
    pub string: u8,
}

pub const DESC_END: u8 = 5;
#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
pub struct EndpointDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}

pub const DESC_HID: u8 = 0x21;
#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
pub struct HIDDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub hid_version: u16,
    pub country_code: u8,
    pub descriptors: u8,
    pub sub_descriptor_type: u8,
    pub sub_descriptor_length: u16,
}
