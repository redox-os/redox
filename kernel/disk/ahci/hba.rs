use arch::memory;

use core::mem::size_of;
use core::u32;

use disk::Disk;

use drivers::io::{Io, Mmio};

use system::error::{Error, Result, EIO};

use super::fis::{FIS_TYPE_REG_H2D, FisRegH2D};

const ATA_CMD_READ_DMA_EXT: u8 = 0x25;
const ATA_CMD_WRITE_DMA_EXT: u8 = 0x35;
const ATA_DEV_BUSY: u8 = 0x80;
const ATA_DEV_DRQ: u8 = 0x08;

const HBA_PORT_CMD_CR: u32 = 1 << 15;
const HBA_PORT_CMD_FR: u32 = 1 << 14;
const HBA_PORT_CMD_FRE: u32 = 1 << 4;
const HBA_PORT_CMD_ST: u32 = 1;
const HBA_PORT_IS_TFES: u32 = 1 << 30;
const HBA_SSTS_PRESENT: u32 = 0x3;
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
    pub clb: Mmio<u64>, // 0x00, command list base address, 1K-byte aligned
    pub fb: Mmio<u64>, // 0x08, FIS base address, 256-byte aligned
    pub is: Mmio<u32>, // 0x10, interrupt status
    pub ie: Mmio<u32>, // 0x14, interrupt enable
    pub cmd: Mmio<u32>, // 0x18, command and status
    pub rsv0: Mmio<u32>, // 0x1C, Reserved
    pub tfd: Mmio<u32>, // 0x20, task file data
    pub sig: Mmio<u32>, // 0x24, signature
    pub ssts: Mmio<u32>, // 0x28, SATA status (SCR0:SStatus)
    pub sctl: Mmio<u32>, // 0x2C, SATA control (SCR2:SControl)
    pub serr: Mmio<u32>, // 0x30, SATA error (SCR1:SError)
    pub sact: Mmio<u32>, // 0x34, SATA active (SCR3:SActive)
    pub ci: Mmio<u32>, // 0x38, command issue
    pub sntf: Mmio<u32>, // 0x3C, SATA notification (SCR4:SNotification)
    pub fbs: Mmio<u32>, // 0x40, FIS-based switch control
    pub rsv1: [Mmio<u32>; 11], // 0x44 ~ 0x6F, Reserved
    pub vendor: [Mmio<u32>; 4], // 0x70 ~ 0x7F, vendor specific
}

impl HbaPort {
    pub fn probe(&self) -> HbaPortType {
        if self.ssts.readf(HBA_SSTS_PRESENT) {
            let sig = self.sig.read();
            match sig {
                HBA_SIG_ATA => HbaPortType::SATA,
                HBA_SIG_ATAPI => HbaPortType::SATAPI,
                HBA_SIG_PM => HbaPortType::PM,
                HBA_SIG_SEMB => HbaPortType::SEMB,
                _ => HbaPortType::Unknown(sig),
            }
        } else {
            HbaPortType::None
        }
    }

    pub fn init(&mut self) {
        self.stop();

        // debugln!("Port Command List");
        let clb = unsafe { memory::alloc_aligned(size_of::<HbaCmdHeader>(), 1024) };
        self.clb.write(clb as u64);

        // debugln!("Port FIS");
        let fb = unsafe { memory::alloc_aligned(256, 256) };
        self.fb.write(fb as u64);

        for i in 0..32 {
            // debugln!("Port Command Table {}", i);
            let cmdheader = unsafe { &mut *(clb as *mut HbaCmdHeader).offset(i) };
            let ctba = unsafe { memory::alloc_aligned(size_of::<HbaCmdTable>(), 256) };
            cmdheader.ctba.write(ctba as u64);
            cmdheader.prdtl.write(0);
        }

        self.start();
    }

    pub fn start(&mut self) {
        // debugln!("Starting port");

        while self.cmd.readf(HBA_PORT_CMD_CR) {}

        self.cmd.writef(HBA_PORT_CMD_FRE, true);
        self.cmd.writef(HBA_PORT_CMD_ST, true);
    }

    pub fn stop(&mut self) {
        // debugln!("Stopping port");

        self.cmd.writef(HBA_PORT_CMD_ST, false);

        while self.cmd.readf(HBA_PORT_CMD_FR | HBA_PORT_CMD_CR) {}

        self.cmd.writef(HBA_PORT_CMD_FRE, false);
    }

    pub fn slot(&self) -> Option<u32> {
        let slots = self.sact.read() | self.ci.read();
        for i in 0..32 {
            if slots & 1 << i == 0 {
                return Some(i);
            }
        }
        None
    }

    pub fn ata_dma_small(&mut self, block: u64, sectors: usize, mut buf: usize, write: bool) -> Result<usize> {
        if buf >= 0x80000000 {
            buf -= 0x80000000;
        }

        // TODO: PRDTL for files larger than 4MB
        let entries = 1;

        if buf > 0 && sectors > 0 {
            self.is.write(u32::MAX);

            if let Some(slot) = self.slot() {
                // debugln!("Slot {}", slot);

                let clb = self.clb.read() as usize;
                let cmdheader = unsafe { &mut *(clb as *mut HbaCmdHeader).offset(slot as isize) };

                cmdheader.cfl.write(((size_of::<FisRegH2D>() / size_of::<u32>()) as u8));
                cmdheader.cfl.writef(1 << 6, write);

                cmdheader.prdtl.write(entries);

                let ctba = cmdheader.ctba.read() as usize;
                unsafe { ::memset(ctba as *mut u8, 0, size_of::<HbaCmdTable>()) };
                let cmdtbl = unsafe { &mut *(ctba as *mut HbaCmdTable) };

                let prdt_entry = &mut cmdtbl.prdt_entry[0];
                prdt_entry.dba.write(buf as u64);
                prdt_entry.dbc.write(((sectors * 512) as u32) | 1);

                let cmdfis = unsafe { &mut *(cmdtbl.cfis.as_ptr() as *mut FisRegH2D) };

                cmdfis.fis_type.write(FIS_TYPE_REG_H2D);
                cmdfis.pm.write(1 << 7);
                if write {
                    cmdfis.command.write(ATA_CMD_WRITE_DMA_EXT);
                } else {
                    cmdfis.command.write(ATA_CMD_READ_DMA_EXT);
                }

                cmdfis.lba0.write(block as u8);
                cmdfis.lba1.write((block >> 8) as u8);
                cmdfis.lba2.write((block >> 16) as u8);

                cmdfis.device.write(1 << 6);

                cmdfis.lba3.write((block >> 24) as u8);
                cmdfis.lba4.write((block >> 32) as u8);
                cmdfis.lba5.write((block >> 40) as u8);

                cmdfis.countl.write(sectors as u8);
                cmdfis.counth.write((sectors >> 8) as u8);

                // debugln!("Busy Wait");
                while self.tfd.readf((ATA_DEV_BUSY | ATA_DEV_DRQ) as u32) {}

                self.ci.writef(1 << slot, true);

                // debugln!("Completion Wait");
                while self.ci.readf(1 << slot) {
                    if self.is.readf(HBA_PORT_IS_TFES) {
                        return Err(Error::new(EIO));
                    }
                }

                if self.is.readf(HBA_PORT_IS_TFES) {
                    return Err(Error::new(EIO));
                }

                Ok(sectors * 512)
            } else {
                debugln!("No Command Slots");
                Err(Error::new(EIO))
            }
        } else {
            debugln!("Invalid request");
            Err(Error::new(EIO))
        }
    }

    pub fn ata_dma(&mut self, block: u64, sectors: usize, buf: usize, write: bool) -> Result<usize> {
        // debugln!("AHCI {:X} DMA BLOCK: {:X} SECTORS: {} BUF: {:X} WRITE: {}", (self as *mut HbaPort) as usize, block, sectors, buf, write);

        if buf > 0 && sectors > 0 {
            let mut sector: usize = 0;
            while sectors - sector >= 255 {
                if let Err(err) = self.ata_dma_small(block + sector as u64, 255, buf + sector * 512, write) {
                    return Err(err);
                }

                sector += 255;
            }
            if sector < sectors {
                if let Err(err) = self.ata_dma_small(block + sector as u64, sectors - sector, buf + sector * 512, write) {
                    return Err(err);
                }
            }

            Ok(sectors * 512)
        } else {
            debugln!("Invalid request");
            Err(Error::new(EIO))
        }
    }
}

#[repr(packed)]
pub struct HbaMem {
    pub cap: Mmio<u32>, // 0x00, Host capability
    pub ghc: Mmio<u32>, // 0x04, Global host control
    pub is: Mmio<u32>, // 0x08, Interrupt status
    pub pi: Mmio<u32>, // 0x0C, Port implemented
    pub vs: Mmio<u32>, // 0x10, Version
    pub ccc_ctl: Mmio<u32>, // 0x14, Command completion coalescing control
    pub ccc_pts: Mmio<u32>, // 0x18, Command completion coalescing ports
    pub em_loc: Mmio<u32>, // 0x1C, Enclosure management location
    pub em_ctl: Mmio<u32>, // 0x20, Enclosure management control
    pub cap2: Mmio<u32>, // 0x24, Host capabilities extended
    pub bohc: Mmio<u32>, // 0x28, BIOS/OS handoff control and status
    pub rsv: [Mmio<u8>; 116], // 0x2C - 0x9F, Reserved
    pub vendor: [Mmio<u8>; 96], // 0xA0 - 0xFF, Vendor specific registers
    pub ports: [HbaPort; 32], // 0x100 - 0x10FF, Port control registers
}

#[repr(packed)]
struct HbaPrdtEntry {
    dba: Mmio<u64>, // Data base address
    rsv0: Mmio<u32>, // Reserved
    dbc: Mmio<u32>, // Byte count, 4M max, interrupt = 1
}

#[repr(packed)]
struct HbaCmdTable {
    // 0x00
    cfis: [Mmio<u8>; 64], // Command FIS

    // 0x40
    acmd: [Mmio<u8>; 16], // ATAPI command, 12 or 16 bytes

    // 0x50
    rsv: [Mmio<u8>; 48], // Reserved

    // 0x80
    prdt_entry: [HbaPrdtEntry; 65536], // Physical region descriptor table entries, 0 ~ 65535
}

#[repr(packed)]
struct HbaCmdHeader {
    // DW0
    cfl: Mmio<u8>, /* Command FIS length in DWORDS, 2 ~ 16, atapi: 4, write - host to device: 2, prefetchable: 1 */
    pm: Mmio<u8>, // Reset - 0x80, bist: 0x40, clear busy on ok: 0x20, port multiplier

    prdtl: Mmio<u16>, // Physical region descriptor table length in entries

    // DW1
    prdbc: Mmio<u32>, // Physical region descriptor byte count transferred

    // DW2, 3
    ctba: Mmio<u64>, // Command table descriptor base address

    // DW4 - 7
    rsv1: [Mmio<u32>; 4], // Reserved
}
