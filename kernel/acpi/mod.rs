use alloc::boxed::Box;
use fs::{KScheme, Resource, Url};
use system::error::{ENOENT, Error, Result};
use system::syscall::O_CREAT;
pub use self::dsdt::DSDT;
pub use self::fadt::FADT;
pub use self::madt::MADT;
pub use self::rsdt::RSDT;
pub use self::sdt::SDTHeader;
pub use self::ssdt::SSDT;

pub mod aml;
pub mod dsdt;
pub mod fadt;
pub mod madt;
pub mod rsdt;
pub mod sdt;
pub mod ssdt;

#[derive(Clone, Debug, Default)]
pub struct Acpi {
    rsdt: RSDT,
    fadt: Option<FADT>,
    dsdt: Option<DSDT>,
    ssdt: Option<SSDT>,
    madt: Option<MADT>,
}

impl Acpi {
    pub fn new() -> Option<Box<Self>> {
        match RSDT::new() {
            Ok(rsdt) => {
                // debugln!("{:#?}", rsdt);

                let mut acpi = box Acpi {
                    rsdt: rsdt,
                    fadt: None,
                    dsdt: None,
                    ssdt: None,
                    madt: None,
                };

                for addr in acpi.rsdt.addrs.iter() {
                    let header = unsafe { &*(*addr as *const SDTHeader) };
                    if let Some(fadt) = FADT::new(header) {
                        // Why does this hang? debugln!("{:#?}", fadt);
                        if let Some(dsdt) = DSDT::new(unsafe {
                            &*(fadt.dsdt as *const SDTHeader)
                        }) {
                            // debugln!("DSDT:");
                            // aml::parse(dsdt.data);
                            acpi.dsdt = Some(dsdt);
                        }
                        acpi.fadt = Some(fadt);
                    } else if let Some(ssdt) = SSDT::new(header) {
                        // debugln!("SSDT:");
                        // aml::parse(ssdt.data);
                        acpi.ssdt = Some(ssdt);
                    } else if let Some(madt) = MADT::new(header) {
                        acpi.madt = Some(madt);
                    } else {
                        for b in header.signature.iter() {
                            debug!("{}", *b as char);
                        }
                        debugln!(": Unknown Table");
                    }
                }

                Some(acpi)
            },
            Err(e) => {
                debugln!("{}", e);
                None
            },
        }
    }
}

impl KScheme for Acpi {
    fn scheme(&self) -> &str {
        "acpi"
    }

    fn open(&mut self, url: Url, flags: usize) -> Result<Box<Resource>> {
        if url.reference() == "off" && flags & O_CREAT == O_CREAT {
            match self.fadt {
                Some(fadt) => {
                    debugln!("Powering Off");
                    unsafe {
                        asm!("out dx, ax" : : "{edx}"(fadt.pm1a_control_block), "{ax}"(0 | 1 << 13) : : "intel", "volatile")
                    };
                },
                None => {
                    debugln!("Unable to power off: No FADT");
                },
            }
        }

        Err(Error::new(ENOENT))
    }
}
