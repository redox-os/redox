use alloc::boxed::Box;

use core::str;

use fs::{KScheme, Resource};
use system::error::{Error, Result, ENOENT};
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
                        //Can't do it debugln!("{:#?}", fadt);
                        if let Some(dsdt) = DSDT::new(unsafe { &*(fadt.dsdt as *const SDTHeader) }) {
                            syslog_debug!("DSDT:");
                            aml::parse(dsdt.data);
                            acpi.dsdt = Some(dsdt);
                        }
                        acpi.fadt = Some(fadt);
                    } else if let Some(ssdt) = SSDT::new(header) {
                        syslog_debug!("SSDT:");
                        aml::parse(ssdt.data);
                        acpi.ssdt = Some(ssdt);
                    } else if let Some(madt) = MADT::new(header) {
                        syslog_debug!("{:#?}", madt);
                        acpi.madt = Some(madt);
                    } else {
                        syslog_debug!("{}: Unknown Table", unsafe { str::from_utf8_unchecked(&header.signature) });
                    }
                }

                Some(acpi)
            }
            Err(e) => {
                debugln!("{}", e);
                None
            }
        }
    }
}

impl KScheme for Acpi {
    fn scheme(&self) -> &'static str {
        "acpi"
    }

    fn open(&mut self, url: &str, flags: usize) -> Result<Box<Resource>> {
        if url.splitn(1, ":").nth(1).unwrap_or("") == "off" && flags & O_CREAT == O_CREAT {
            match self.fadt {
                Some(fadt) => {
                    debugln!("Powering Off");
                    unsafe {
                        asm!("out dx, ax" : : "{edx}"(fadt.pm1a_control_block), "{ax}"(0 | 1 << 13) : : "intel", "volatile")
                    };
                }
                None => {
                    debugln!("Unable to power off: No FADT");
                }
            }
        }

        Err(Error::new(ENOENT))
    }
}
