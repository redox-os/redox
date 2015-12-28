use alloc::boxed::Box;

use core::intrinsics::atomic_singlethreadfence;
use core::u32;

use drivers::pciconfig::PciConfig;

use schemes::KScheme;

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

const FIS_TYPE_REG_H2D: u8 = 0x27;	// Register FIS - host to device
const FIS_TYPE_REG_D2H: u8 = 0x34;	// Register FIS - device to host
const FIS_TYPE_DMA_ACT: u8 = 0x39;	// DMA activate FIS - device to host
const FIS_TYPE_DMA_SETUP: u8 = 0x41;	// DMA setup FIS - bidirectional
const FIS_TYPE_DATA: u8 = 0x46;	// Data FIS - bidirectional
const FIS_TYPE_BIST: u8 = 0x58;	// BIST activate FIS - bidirectional
const FIS_TYPE_PIO_SETUP: u8 = 0x5F;	// PIO setup FIS - device to host
const FIS_TYPE_DEV_BITS: u8 = 0xA1;	// Set device bits FIS - device to host

#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
struct FisRegH2D{
	// DWORD 0
	fis_type: u8,	// FIS_TYPE_REG_H2D

	pm: u8,	       // Port multiplier, 1: Command, 0: Control

	command: u8,	// Command register
	featurel: u8,	// Feature register, 7:0

	// DWORD 1
	lba0: u8,		// LBA low register, 7:0
	lba1: u8,		// LBA mid register, 15:8
	lba2: u8,		// LBA high register, 23:16
	device: u8,		// Device register

	// DWORD 2
	lba3: u8,		// LBA register, 31:24
	lba4: u8,		// LBA register, 39:32
	lba5: u8,		// LBA register, 47:40
	featureh: u8,	// Feature register, 15:8

	// DWORD 3
	countl: u8,		// Count register, 7:0
	counth: u8,		// Count register, 15:8
	icc: u8,		// Isochronous command completion
	control: u8,	// Control register

	// DWORD 4
	rsv1: [u8; 4],  // Reserved
}

#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
struct FisRegD2H {
	// DWORD 0
	fis_type: u8,    // FIS_TYPE_REG_D2H

	pm: u8,          // Port multiplier, Interrupt bit: 2

	status: u8,      // Status register
	error: u8,       // Error register

	// DWORD 1
	lba0: u8,        // LBA low register, 7:0
	lba1: u8,        // LBA mid register, 15:8
	lba2: u8,        // LBA high register, 23:16
	device: u8,      // Device register

	// DWORD 2
	lba3: u8,        // LBA register, 31:24
	lba4: u8,        // LBA register, 39:32
	lba5: u8,        // LBA register, 47:40
	rsv2: u8,        // Reserved

	// DWORD 3
	countl: u8,      // Count register, 7:0
	counth: u8,      // Count register, 15:8
	rsv3: [u8; 2],   // Reserved

	// DWORD 4
	rsv4: [u8; 4],   // Reserved
}

#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
struct FisData {
	// DWORD 0
	fis_type: u8,	// FIS_TYPE_DATA

	pm: u8,	        // Port multiplier

	rsv1: [u8; 2],	// Reserved

	// DWORD 1 ~ N
	data: u32,      // Payload
}

#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
struct FisPioSetup {
	// DWORD 0
	fis_type: u8,	// FIS_TYPE_PIO_SETUP

	pm: u8,      	// Port multiplier, direction: 4 - device to host, interrupt: 2

	status: u8,		// Status register
	error: u8,		// Error register

	// DWORD 1
	lba0: u8,		// LBA low register, 7:0
	lba1: u8,		// LBA mid register, 15:8
	lba2: u8,		// LBA high register, 23:16
	device: u8,		// Device register

	// DWORD 2
	lba3: u8,		// LBA register, 31:24
	lba4: u8,		// LBA register, 39:32
	lba5: u8,		// LBA register, 47:40
	rsv2: u8,		// Reserved

	// DWORD 3
	countl: u8,		// Count register, 7:0
	counth: u8,		// Count register, 15:8
	rsv3: u8,		// Reserved
	e_status: u8,	// New value of status register

	// DWORD 4
	tc: u16,		// Transfer count
	rsv4: [u8; 2],	// Reserved
}

#[repr(packed)]
#[derive(Copy, Clone, Debug, Default)]
struct FisDmaSetup {
	// DWORD 0
	fis_type: u8,	     // FIS_TYPE_DMA_SETUP

	pm: u8,	             // Port multiplier, direction: 4 - device to host, interrupt: 2, auto-activate: 1

    rsv1: [u8; 2],       // Reserved

	//DWORD 1&2
    DMAbufferID: u64,    // DMA Buffer Identifier. Used to Identify DMA buffer in host memory. SATA Spec says host specific and not in Spec. Trying AHCI spec might work.

    //DWORD 3
    rsv3: u32,           //More reserved

    //DWORD 4
    DMAbufOffset: u32,   //Byte offset into buffer. First 2 bits must be 0

    //DWORD 5
    TransferCount: u32,  //Number of bytes to transfer. Bit 0 must be 0

    //DWORD 6
    rsv6: u32,          //Reserved
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
