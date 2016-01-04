use core::ptr;

use super::SDTHeader;

#[repr(packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SSDT {
    pub header: SDTHeader,
    pub data: &'static [u8]

}

impl SSDT {
    pub fn new(header: *const SDTHeader) -> Option<Self> {
        let header = unsafe { *header };
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
