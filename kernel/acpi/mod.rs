use alloc::boxed::Box;
use schemes::{Result, KScheme, Resource, Url};
use syscall::{SysError, O_CREAT, ENOENT};
pub use self::dsdt::DSDT;
pub use self::fadt::FADT;
pub use self::rsdt::RSDT;
pub use self::sdt::SDTHeader;
pub use self::ssdt::SSDT;

pub mod aml;
pub mod dsdt;
pub mod fadt;
pub mod rsdt;
pub mod sdt;
pub mod ssdt;

#[derive(Clone, Debug, Default)]
pub struct Acpi {
    rsdt: RSDT,
    fadt: Option<FADT>,
    dsdt: Option<DSDT>,
    ssdt: Option<SSDT>,
}

impl Acpi {
    pub fn new() -> Option<Box<Self>> {
        match RSDT::new() {
            Ok(rsdt) => {
                //debugln!("{:#?}", rsdt);

                let mut acpi = box Acpi {
                    rsdt: rsdt,
                    fadt: None,
                    dsdt: None,
                    ssdt: None,
                };

                for addr in acpi.rsdt.addrs.iter() {
                    let header = *addr as *const SDTHeader;

                    if let Some(fadt) = FADT::new(header) {
                        //Why does this hang? debugln!("{:#?}", fadt);
                        if let Some(dsdt) = DSDT::new(fadt.Dsdt as *const SDTHeader) {
                            //debugln!("DSDT:");
                            //aml::parse(dsdt.data);
                            acpi.dsdt = Some(dsdt);
                        }
                        acpi.fadt = Some(fadt);
                    } else if let Some(ssdt) = SSDT::new(header) {
                        //debugln!("SSDT:");
                        //aml::parse(ssdt.data);
                        acpi.ssdt = Some(ssdt);
                    } else {
                        //debugln!("{:X}: {:#?}", addr, unsafe { *header });
                    }
                }

                Some(acpi)
            },
            Err(e) => {
                debugln!("{}", e);
                None
            }
        }
    }
}

impl KScheme for Acpi {
    fn scheme(&self) -> &str {
        "acpi"
    }

    fn open(&mut self, url: &Url, flags: usize) -> Result<Box<Resource>> {
        if url.reference() == "off" && flags & O_CREAT == O_CREAT {
            match self.fadt {
                Some(fadt) => {
                    debugln!("Powering Off");
                    unsafe { asm!("out dx, ax" : : "{edx}"(fadt.PM1aControlBlock), "{ax}"(0 | 1 << 13) : : "intel", "volatile") };
                },
                None => {
                    debugln!("Unable to power off: No FADT");
                }
            }
        }

        Err(SysError::new(ENOENT))
    }
}
