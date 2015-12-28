use core::intrinsics::atomic_singlethreadfence;
use core::u32;

const HBA_PxCMD_CR: u32 = 1 << 15;
const HBA_PxCMD_FR: u32 = 1 << 14;
const HBA_PxCMD_FRE: u32 = 1 << 4;
const HBA_PxCMD_ST: u32 = 1;

#[repr(packed)]
pub struct HbaPort {
    pub clb: u32,
    pub clbu: u32,
    pub fb: u32,
    pub fbu: u32,
    pub is: u32,
    pub ie: u32,
    pub cmd: u32,
    pub rsv0: u32,
    pub tfd: u32,
    pub sig: u32,
    pub ssts: u32,
    pub sctl: u32,
    pub serr: u32,
    pub sact: u32,
    pub ci: u32,
    pub sntf: u32,
    pub fbs: u32,
    pub rsv1: [u32; 11],
    pub vendor: [u32; 4]
}

#[derive(Debug)]
pub enum HbaPortType {
    None,
    Unknown(u32),
    SATA,
    SATAPI,
    PM,
    SEMB,
}

const HBA_PORT_PRESENT: u32 = 0x13;
const SATA_SIG_ATA: u32 = 0x00000101;
const SATA_SIG_ATAPI: u32 = 0xEB140101;
const SATA_SIG_PM: u32 = 0x96690101;
const SATA_SIG_SEMB: u32 = 0xC33C0101;

impl HbaPort {
    pub fn probe(&self) -> HbaPortType {
        if self.ssts & HBA_PORT_PRESENT != HBA_PORT_PRESENT {
            HbaPortType::None
        } else {
            match self.sig {
                SATA_SIG_ATA => HbaPortType::SATA,
                SATA_SIG_ATAPI => HbaPortType::SATAPI,
                SATA_SIG_PM => HbaPortType::PM,
                SATA_SIG_SEMB => HbaPortType::SEMB,
                _ => HbaPortType::Unknown(self.sig)
            }
        }
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
    pub cap: u32,
    pub ghc: u32,
    pub is: u32,
    pub pi: u32,
    pub vs: u32,
    pub ccc_ctl: u32,
    pub ccc_pts: u32,
    pub em_loc: u32,
    pub em_ctl: u32,
    pub cap2: u32,
    pub bohc: u32,
    pub rsv: [u8; 116],
    pub vendor: [u8; 96],
    pub ports: [HbaPort; 32]
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
struct HbaCmdTable <T> {
	// 0x00
	cfis: [u8; 64],	// Command FIS

	// 0x40
	acmd: [u8; 16],	// ATAPI command, 12 or 16 bytes

	// 0x50
	rsv: [u8; 48],	// Reserved

	// 0x80
	prdt_entry: T,	// Physical region descriptor table entries, 0 ~ 65535
}
