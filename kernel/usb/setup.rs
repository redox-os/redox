#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Setup {
    pub request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub len: u16,
}

impl Setup {
    pub fn get_status() -> Setup {
        Setup {
            request_type: 0b10000000,
            request: 0x00,
            value: 0,
            index: 0,
            len: 2,
        }
    }

    pub fn clear_feature(feature: u16) -> Setup {
        Setup {
            request_type: 0b00000000,
            request: 0x01,
            value: feature,
            index: 0,
            len: 0,
        }
    }

    pub fn set_feature(feature: u16) -> Setup {
        Setup {
            request_type: 0b00000000,
            request: 0x03,
            value: feature,
            index: 0,
            len: 0,
        }
    }

    pub fn set_address(address: u8) -> Setup {
        Setup {
            request_type: 0b00000000,
            request: 0x05,
            value: (address as u16) & 0x7F,
            index: 0,
            len: 0,
        }
    }

    pub fn get_descriptor(descriptor_type: u8,
                          descriptor_index: u8,
                          language_id: u16,
                          descriptor_len: u16)
                          -> Setup {
        Setup {
            request_type: 0b10000000,
            request: 0x06,
            value: (descriptor_type as u16) << 8 | (descriptor_index as u16),
            index: language_id,
            len: descriptor_len,
        }
    }

    pub fn set_descriptor(descriptor_type: u8,
                          descriptor_index: u8,
                          language_id: u16,
                          descriptor_len: u16)
                          -> Setup {
        Setup {
            request_type: 0b00000000,
            request: 0x07,
            value: (descriptor_type as u16) << 8 | (descriptor_index as u16),
            index: language_id,
            len: descriptor_len,
        }
    }

    pub fn get_configuration() -> Setup {
        Setup {
            request_type: 0b10000000,
            request: 0x08,
            value: 0,
            index: 0,
            len: 1,
        }
    }

    pub fn set_configuration(value: u8) -> Setup {
        Setup {
            request_type: 0b00000000,
            request: 0x09,
            value: value as u16,
            index: 0,
            len: 0,
        }
    }
}
