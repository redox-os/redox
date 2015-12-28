use super::SDTHeader;


#[repr(packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DSDT {
    pub header: SDTHeader,
    pub data: &'static [u8]

}

impl DSDT {
    pub fn new(header: *const SDTHeader) -> Option<Self> {
        if unsafe { (*header).valid("DSDT") } {
            Some(DSDT {
                header: unsafe { (*header).clone() },
                data: unsafe { (*header).data() }
            })
        } else {
            None
        }
    }
}
