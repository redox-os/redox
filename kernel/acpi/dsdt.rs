use super::SDTHeader;

#[repr(packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DSDT {
    pub header: SDTHeader,
    pub data: &'static [u8]

}

impl DSDT {
    pub fn new(header: *const SDTHeader) -> Option<Self> {
        let header = unsafe { *header };
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
