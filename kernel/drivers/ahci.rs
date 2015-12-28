use alloc::boxed::Box;

use core::intrinsics::atomic_singlethreadfence;
use core::u32;

use schemes::KScheme;

use super::pciconfig::PciConfig;

const HBA_PxCMD_CR: u32 = 1 << 15;
const HBA_PxCMD_FR: u32 = 1 << 14;
const HBA_PxCMD_FRE: u32 = 1 << 4;
const HBA_PxCMD_ST: u32 = 1;

#[repr(packed)]
struct HBAPort {
    clb: u32,
    clbu: u32,
    fb: u32,
    fbu: u32,
    is: u32,
    ie: u32,
    cmd: u32,
    rsv0: u32,
    tfd: u32,
    sig: u32,
    ssts: u32,
    sctl: u32,
    serr: u32,
    sact: u32,
    ci: u32,
    sntf: u32,
    fbs: u32,
    rsv1: [u32; 11],
    vendor: [u32; 4]
}

#[derive(Debug)]
enum HBAPortType {
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

impl HBAPort {
    pub fn probe(&self) -> HBAPortType {
        if self.ssts & HBA_PORT_PRESENT != HBA_PORT_PRESENT {
            HBAPortType::None
        } else {
            match self.sig {
                SATA_SIG_ATA => HBAPortType::SATA,
                SATA_SIG_ATAPI => HBAPortType::SATAPI,
                SATA_SIG_PM => HBAPortType::PM,
                SATA_SIG_SEMB => HBAPortType::SEMB,
                _ => HBAPortType::Unknown(self.sig)
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
}

#[repr(packed)]
struct HBAMem {
    cap: u32,
    ghc: u32,
    is: u32,
    pi: u32,
    vs: u32,
    ccc_ctl: u32,
    ccc_pts: u32,
    em_loc: u32,
    em_ctl: u32,
    cap2: u32,
    bohc: u32,
    rsv: [u8; 116],
    vendor: [u8; 96],
    ports: [HBAPort; 32]
}

pub struct Ahci {
    pci: PciConfig,
    mem: *mut HBAMem,
    irq: u8,
}

impl Ahci {
    pub fn new(mut pci: PciConfig) -> Box<Self> {
        let base = unsafe { (pci.read(0x24) & 0xFFFFFFF0) as usize };
        let irq = unsafe { (pci.read(0x3C) & 0xF) as u8 };

        let mut module = box Ahci {
            pci: pci,
            mem: base as *mut HBAMem,
            irq: irq,
        };

        module.init();

        module
    }

    fn init(&mut self) {
        debugln!("AHCI on: {:X} IRQ: {:X}", self.mem as usize, self.irq);

        let mem = unsafe { &mut * self.mem };

        for i in 0..32 {
            if mem.pi & 1 << i == 1 << i {
                debugln!("Port {}: {:X} {:?}", i, mem.ports[i].ssts, mem.ports[i].probe());
            }
        }
    }
}

impl KScheme for Ahci {
    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            debugln!("AHCI IRQ");
        }
    }

    fn on_poll(&mut self) {
    }
}
