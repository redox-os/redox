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
struct FisData<T> {
	// DWORD 0
	fis_type: u8,	// FIS_TYPE_DATA

	pm: u8,	        // Port multiplier

	rsv1: [u8; 2],	// Reserved

	// DWORD 1 ~ N
	data: T,      // Payload
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
