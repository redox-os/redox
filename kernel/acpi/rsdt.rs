use core::mem::size_of;

use super::SDTHeader;

#[repr(packed)]
#[derive(Clone, Copy, Debug, Default)]
struct RSDP {
    signature: [u8; 8],
    checksum: u8,
    oemid: [u8; 6],
    revision: u8,
    addr: u32,
}

const SIGNATURE: &'static [u8] = b"RSD PTR ";

impl RSDP {
    pub fn new() -> Result<Self, &'static str> {
        // Search top of bios region
        let mut search_ptr = 0xE0000;
        while search_ptr < 0xFFFFF {
            let rsdp = search_ptr as *const RSDP;
            if unsafe { (*rsdp).valid() } {
                return Ok(unsafe { *rsdp });
            }
            search_ptr += 16;
        }

        Err("Did not find RSDP")
    }

    // TODO: Checksum validation
    pub fn valid(&self) -> bool {
        if self.signature == SIGNATURE {
            let ptr = (self as *const Self) as *const u8;
            let sum = (0..size_of::<Self>() as isize)
                .fold(0, |sum, i| sum + unsafe { *ptr.offset(i) });

            sum == 0
        } else {
            false
        }
    }
}

#[repr(packed)]
#[derive(Clone, Debug, Default)]
pub struct RSDT {
    pub header: SDTHeader,
    pub addrs: &'static [u32],
}

impl RSDT {
    pub fn new() -> Result<Self, &'static str> {
        match RSDP::new() {
            Ok(rsdp) => {
                let header = rsdp.addr as *const SDTHeader;
                if unsafe { (*header).valid("RSDT") } {
                    Ok(RSDT {
                        header: unsafe { *header },
                        addrs: unsafe { (*header).data() },
                    })
                } else {
                    Err("Did not find RSDT")
                }
            },
            Err(e) => Err(e),
        }
    }
}
