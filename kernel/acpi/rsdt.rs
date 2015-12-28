use core::mem::size_of;

use super::SDTHeader;

#[repr(packed)]
#[derive(Clone, Copy, Debug, Default)]
struct RSDP {
    signature: [u8; 8],
    checksum: u8,
    oemid: [u8; 6],
    revision: u8,
    addr: u32
}

const SIGNATURE: &'static [u8] = b"RSD PTR ";

impl RSDP {
    pub fn new() -> Option<Self> {
        //Search top of bios region
        let mut search_ptr = 0xE0000;
        while search_ptr < 0xFFFFF {
            let rsdp = search_ptr as *const RSDP;
            if unsafe { (*rsdp).valid() } {
                return Some(unsafe { (*rsdp).clone() });
            }
            search_ptr += 16;
        }

        None
    }

    //TODO: Checksum validation
    pub fn valid(&self) -> bool {
        if self.signature == SIGNATURE {
            let mut sum = 0;

            let ptr = (self as *const Self) as *const u8;
            for i in 0..size_of::<Self>() as isize {
                sum += unsafe { *ptr.offset(i) }
            }

            sum == 0
        } else {
            false
        }
    }
}

#[repr(packed)]
#[derive(Debug)]
pub struct RSDT {
    pub header: SDTHeader,
    pub addrs: &'static [u32]
}

impl RSDT {
    pub fn new() -> Option<Self> {
        match RSDP::new() {
            Some(rsdp) => {
                let header = rsdp.addr as *const SDTHeader;
                if unsafe { (*header).valid("RSDT") } {
                    Some(RSDT {
                        header: unsafe { (*header).clone() },
                        addrs: unsafe { (*header).data() }
                    })
                } else {
                    None
                }
            },
            None => {
                debugln!("Did not find RSDP");
                None
            }
        }
    }
}
