use super::SDTHeader;

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct SSDT {
    pub header: &'static SDTHeader,
    pub data: &'static [u8],
}

impl SSDT {
    pub fn new(header: &'static SDTHeader) -> Option<Self> {
        if header.valid("SSDT") {
            Some(SSDT {
                header: header,
                data: header.data(),
            })
        } else {
            None
        }
    }
}
