/// Common PCI constants from the PCI spec
pub mod config {
    /// Legacy PCI Configuration Space Access registers
    pub const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
    pub const PCI_CONFIG_DATA: u16 = 0xCFC;
    pub const PCI_CONFIG_ADDRESS_ENABLE: u32 = 0x80000000;
    pub const PCI_BUS_OFFSET: u32 = 16;
    pub const PCI_SLOT_OFFSET: u32 = 11;
    pub const PCI_FUNC_OFFSET: u32 = 8;

    /// PCI Configuration Space registers (256 bytes window)
    /// In theory 32 bit aligned access only.
    /// In practice, however, things seem to be working even with unaligned
    /// access.
    pub const PCI_CFG_VENDOR_ID: u8 = 0x00;
    pub const PCI_CFG_DEVICE_ID: u8 = 0x02;
    pub const PCI_CFG_COMMAND: u8 = 0x04;
    pub const PCI_CFG_STATUS: u8 = 0x06;
    pub const PCI_CFG_REVISION_ID: u8 = 0x08;
    pub const PCI_CFG_PROG_INTERFACE: u8 = 0x09;
    pub const PCI_CFG_SUBCLASS: u8 = 0x0A;
    pub const PCI_CFG_BASECLASS: u8 = 0x0B;
    pub const PCI_CFG_CACHELINE_SIZE: u8 = 0x0C;
    pub const PCI_CFG_LATENCY_TIMER: u8 = 0x0D;
    pub const PCI_CFG_HEADER_TYPE: u8 = 0x0E;
    pub const PCI_CFG_BIST: u8 = 0x0F;
    pub const PCI_CFG_BAR_1: u8 = 0x10;
    pub const PCI_CFG_BAR_2: u8 = 0x14;
    pub const PCI_CFG_BAR_3: u8 = 0x18;
    pub const PCI_CFG_BAR_4: u8 = 0x1C;
    pub const PCI_CFG_BAR_5: u8 = 0x20;
    pub const PCI_CFG_BAR_6: u8 = 0x24;
    pub const PCI_CFG_CARDBUS_CIS_PTR: u8 = 0x28;
    pub const PCI_CFG_SUBSYSTEM_VENDOR_ID: u8 = 0x2C;
    pub const PCI_CFG_SUBSYSTEM_ID: u8 = 0x2E;
    pub const PCI_CFG_CAPABILITIES_PTR: u8 = 0x34;
    pub const PCI_CFG_INTERRUPT_LINE: u8 = 0x3C;
    pub const CI_CFG_INTERRUPT_PIN: u8 = 0x3D;
}

pub mod class {
    /// PCI Class Codes
    pub const NONE: u8 = 0x00;
    pub const MASS_STORAGE: u8 = 0x01;
    pub const NETWORK: u8 = 0x02;
    pub const DISPLAY: u8 = 0x03;
    pub const MULTIMEDIA: u8 = 0x04;
    pub const MEMORY: u8 = 0x05;
    pub const BRIDGE_DEVICE: u8 = 0x06;
    pub const COMMUNICATION: u8 = 0x07;
    pub const SYSTEM_PERIPHERALS: u8 = 0x08;
    pub const INPUT: u8 = 0x09;
    pub const DOCKING_STATION: u8 = 0x0A;
    pub const PROCESSOR: u8 = 0x0B;
    pub const SERIAL_BUS: u8 = 0x0C;
    pub const WIRELESS: u8 = 0x0D;
    pub const INTELLIGENT_IO: u8 = 0x0E;
    pub const SATTELITE_COMMUNICATION: u8 = 0x0F;
    pub const ENCRYPTION: u8 = 0x10;
    pub const DATA_ACQUISITION: u8 = 0x11;
    // Class codes 0x12 - 0xFE are reserved
    pub const OTHER: u8 = 0xFF;
}

pub mod subclass {
    /// PCI Mass Storage Subclass Codes
    pub const SCSI: u8 = 0x00;
    pub const IDE: u8 = 0x01;
    pub const FLOPPY: u8 = 0x02;
    pub const IPI: u8 = 0x03;
    pub const RAID: u8 = 0x04;
    pub const ATA: u8 = 0x05;
    pub const SATA: u8 = 0x06;
    pub const SAS: u8 = 0x07;
    pub const NVM: u8 = 0x08;

    /// PCI Network Subclass Codes
    pub const ETHERNET: u8 = 0x00;
    pub const INFINIBAND: u8 = 0x07;
    pub const FABRIC: u8 = 0x08;

    /// PCI Display Subclass Codes
    pub const VGA: u8 = 0x00;
    pub const XGA: u8 = 0x01;

    /// PCI Serial Bus Subclass Codes
    pub const FIREWIRE: u8 = 0x00;
    pub const USB: u8 = 0x03;
}

pub mod programming_interface {
    /// PCI SATA Programming Interface
    pub const AHCI: u8 = 0x01;

    /// PCI USB Programming Interface
    pub const UHCI: u8 = 0x00;
    pub const OHCI: u8 = 0x10;
    pub const EHCI: u8 = 0x20;
    pub const XHCI: u8 = 0x30;
}

pub mod vendorid {
    pub const INTEL: u16 = 0x8086;
    pub const REALTEK: u16 = 0x10EC;
    pub const REDHAT: u16 = 0x1AF4;
    pub const ILLEGAL: u16 = 0xFFFF;
}

pub mod deviceid {
    // Realtek
    pub const RTL8139: u16 = 0x8139;        // RTL-8100/8101L/8139 PCI Fast Ethernet Adapter

    // Intel
    pub const GBE_82540EM: u16 = 0x100E;    // 82540EM Gigabit Ethernet Controller
    pub const AC97_82801AA: u16 = 0x2415;   // 82801AA AC'97 Audio Controller
    pub const AC97_ICH4: u16 = 0x24C5;      // 82801DB/DBL/DBM (ICH4/ICH4-L/ICH4-M) AC'97 Audio
    pub const INTELHDA_ICH6: u16 = 0x2668;  // 82801FB/FBM/FR/FW/FRW High Definition Audio
}
