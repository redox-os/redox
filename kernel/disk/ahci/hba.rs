use core::intrinsics::atomic_singlethreadfence;
use core::u32;

const HBA_PxCMD_CR: u32 = 1 << 15;
const HBA_PxCMD_FR: u32 = 1 << 14;
const HBA_PxCMD_FRE: u32 = 1 << 4;
const HBA_PxCMD_ST: u32 = 1;
const HBA_SSTS_PRESENT: u32 = 0x13;
const HBA_SIG_ATA: u32 = 0x00000101;
const HBA_SIG_ATAPI: u32 = 0xEB140101;
const HBA_SIG_PM: u32 = 0x96690101;
const HBA_SIG_SEMB: u32 = 0xC33C0101;

#[derive(Debug)]
pub enum HbaPortType {
    None,
    Unknown(u32),
    SATA,
    SATAPI,
    PM,
    SEMB,
}

#[repr(packed)]
pub struct HbaPort {
    pub clb: u32,   // 0x00, command list base address, 1K-byte aligned
    pub clbu: u32,  // 0x04, command list base address upper 32 bits
    pub fb: u32,    // 0x08, FIS base address, 256-byte aligned
    pub fbu: u32,   // 0x0C, FIS base address upper 32 bits
    pub is: u32,    // 0x10, interrupt status
    pub ie: u32,    // 0x14, interrupt enable
    pub cmd: u32,   // 0x18, command and status
    pub rsv0: u32,  // 0x1C, Reserved
    pub tfd: u32,   // 0x20, task file data
    pub sig: u32,   // 0x24, signature
    pub ssts: u32,  // 0x28, SATA status (SCR0:SStatus)
    pub sctl: u32,  // 0x2C, SATA control (SCR2:SControl)
    pub serr: u32,  // 0x30, SATA error (SCR1:SError)
    pub sact: u32,  // 0x34, SATA active (SCR3:SActive)
    pub ci: u32,    // 0x38, command issue
    pub sntf: u32,  // 0x3C, SATA notification (SCR4:SNotification)
    pub fbs: u32,   // 0x40, FIS-based switch control
    pub rsv1: [u32; 11],    // 0x44 ~ 0x6F, Reserved
    pub vendor: [u32; 4]    // 0x70 ~ 0x7F, vendor specific
}

impl HbaPort {
    pub fn probe(&self) -> HbaPortType {
        if self.ssts & HBA_SSTS_PRESENT != HBA_SSTS_PRESENT {
            HbaPortType::None
        } else {
            match self.sig {
                HBA_SIG_ATA => HbaPortType::SATA,
                HBA_SIG_ATAPI => HbaPortType::SATAPI,
                HBA_SIG_PM => HbaPortType::PM,
                HBA_SIG_SEMB => HbaPortType::SEMB,
                _ => HbaPortType::Unknown(self.sig)
            }
        }
    }

    pub fn init(&mut self) {
    }

    pub fn start(&mut self) {
        loop {
            if self.cmd & HBA_PxCMD_CR == 0 {
                break;
            }
            unsafe { atomic_singlethreadfence() };
        }

        self.cmd |= HBA_PxCMD_FRE;
        self.cmd |= HBA_PxCMD_ST;
    }

    pub fn stop(&mut self) {
    	self.cmd &= u32::MAX - HBA_PxCMD_ST;

    	loop {
    		if self.cmd & (HBA_PxCMD_FR | HBA_PxCMD_CR) == 0 {
                break;
            }
            unsafe { atomic_singlethreadfence() };
    	}

    	self.cmd &= u32::MAX - HBA_PxCMD_FRE;
    }

    pub fn slot(&self) -> Option<u32> {
        let slots = self.sact | self.ci;
        for i in 0..32 {
            if slots & 1 << i == 0 {
                return Some(i);
            }
        }
        None
    }
}

#[repr(packed)]
pub struct HbaMem {
    pub cap: u32,       // 0x00, Host capability
    pub ghc: u32,       // 0x04, Global host control
    pub is: u32,        // 0x08, Interrupt status
    pub pi: u32,        // 0x0C, Port implemented
    pub vs: u32,        // 0x10, Version
    pub ccc_ctl: u32,   // 0x14, Command completion coalescing control
    pub ccc_pts: u32,   // 0x18, Command completion coalescing ports
    pub em_loc: u32,    // 0x1C, Enclosure management location
    pub em_ctl: u32,    // 0x20, Enclosure management control
    pub cap2: u32,      // 0x24, Host capabilities extended
    pub bohc: u32,      // 0x28, BIOS/OS handoff control and status
    pub rsv: [u8; 116],         // 0x2C - 0x9F, Reserved
    pub vendor: [u8; 96],       // 0xA0 - 0xFF, Vendor specific registers
    pub ports: [HbaPort; 32]    // 0x100 - 0x10FF, Port control registers
}

#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
struct HbaPrdtEntry {
   dba: u32,		// Data base address
   dbau: u32,		// Data base address upper 32 bits
   rsv0: u32,		// Reserved
   dbc: u32,		// Byte count, 4M max, interrupt = 1
}

#[repr(packed)]
struct HbaCmdTable {
	// 0x00
	cfis: [u8; 64],	// Command FIS

	// 0x40
	acmd: [u8; 16],	// ATAPI command, 12 or 16 bytes

	// 0x50
	rsv: [u8; 48],	// Reserved

	// 0x80
	prdt_entry: [HbaPrdtEntry; 65536],	// Physical region descriptor table entries, 0 ~ 65535
}

#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
struct HbaCmdHeader {
	// DW0
	cfl: u8,		// Command FIS length in DWORDS, 2 ~ 16, atapi: 4, write - host to device: 2, prefetchable: 1
	pm: u8,		    // Reset - 0x80, bist: 0x40, clear busy on ok: 0x20, port multiplier

	prdtl: u16,		// Physical region descriptor table length in entries

	// DW1
	prdbc: u32,		// Physical region descriptor byte count transferred

	// DW2, 3
	ctba: u32,		// Command table descriptor base address
	ctbau: u32,		// Command table descriptor base address upper 32 bits

	// DW4 - 7
	rsv1: [u32; 4],	// Reserved
}
