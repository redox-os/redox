use super::SDTHeader;
use core::ptr;

#[repr(packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GenericAddressStructure {
    address_space: u8,
    bit_width: u8,
    bit_offset: u8,
    access_size: u8,
    address: u64,
}

#[repr(packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct FADT {
    pub header: SDTHeader,
    pub FirmwareCtrl: u32,
    pub Dsdt: u32,

    // field used in ACPI 1.0; no longer in use, for compatibility only
    Reserved: u8,

    // TODO Proper naming after the conventions
    pub PreferredPowerManagementProfile: u8,
    pub SCI_Interrupt: u16,
    pub SMI_CommandPort: u32,
    pub AcpiEnable: u8,
    pub AcpiDisable: u8,
    pub S4BIOS_REQ: u8,
    pub PSTATE_Control: u8,
    pub PM1aEventBlock: u32,
    pub PM1bEventBlock: u32,
    pub PM1aControlBlock: u32,
    pub PM1bControlBlock: u32,
    pub PM2ControlBlock: u32,
    pub PMTimerBlock: u32,
    pub GPE0Block: u32,
    pub GPE1Block: u32,
    pub PM1EventLength: u8,
    pub PM1ControlLength: u8,
    pub PM2ControlLength: u8,
    pub PMTimerLength: u8,
    pub GPE0Length: u8,
    pub GPE1Length: u8,
    pub GPE1Base: u8,
    pub CStateControl: u8,
    pub WorstC2Latency: u16,
    pub WorstC3Latency: u16,
    pub FlushSize: u16,
    pub FlushStride: u16,
    pub DutyOffset: u8,
    pub DutyWidth: u8,
    pub DayAlarm: u8,
    pub MonthAlarm: u8,
    pub Century: u8,

    // reserved in ACPI 1.0; used since ACPI 2.0+
    pub BootArchitectureFlags: u16,

    Reserved2: u8,
    pub Flags: u32,

    // 12 byte structure; see below for details
    pub ResetReg: GenericAddressStructure,

    pub ResetValue: u8,
    Reserved3: [u8; 3],

    // 64bit pointers - Available on ACPI 2.0+
    pub X_FirmwareControl: u64,
    pub X_Dsdt: u64,

    pub X_PM1aEventBlock: GenericAddressStructure,
    pub X_PM1bEventBlock: GenericAddressStructure,
    pub X_PM1aControlBlock: GenericAddressStructure,
    pub X_PM1bControlBlock: GenericAddressStructure,
    pub X_PM2ControlBlock: GenericAddressStructure,
    pub X_PMTimerBlock: GenericAddressStructure,
    pub X_GPE0Block: GenericAddressStructure,
    pub X_GPE1Block: GenericAddressStructure,
}

impl FADT {
    pub fn new(header: *const SDTHeader) -> Option<Self> {
        if unsafe { (*header).valid("FACP") } {
            Some(unsafe { ptr::read(header as *const FADT) })
        } else {
            None
        }
    }
}
