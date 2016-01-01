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
        if unsafe { (*header).valid("SSDT") } {
            Some(SSDT {
                header: unsafe { *header },
                data: unsafe { (*header).data() }
            })
        } else {
            None
        }
    }
}
