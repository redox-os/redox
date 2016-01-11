use super::SDTHeader;

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct DSDT {
    pub header: &'static SDTHeader,
    pub data: &'static [u8],
}

impl DSDT {
    pub fn new(header: &'static SDTHeader) -> Option<Self> {
        if header.valid("DSDT") {
            Some(DSDT {
                header: header,
                data: header.data(),
            })
        } else {
            None
        }
    }
}
