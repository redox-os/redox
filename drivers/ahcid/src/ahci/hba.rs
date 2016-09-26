use io::{Io, Mmio};

use std::mem::size_of;
use std::{ptr, u32};

use syscall::{physalloc, physmap, physunmap, virttophys, MAP_WRITE};
use syscall::error::{Error, Result, EIO};

use super::fis::{FisType, FisRegH2D};

const ATA_CMD_READ_DMA_EXT: u8 = 0x25;
const ATA_CMD_WRITE_DMA_EXT: u8 = 0x35;
const ATA_CMD_IDENTIFY: u8 = 0xEC;
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

        let clb_phys = unsafe { physalloc(size_of::<HbaCmdHeader>()).unwrap() };
        self.clb.write(clb_phys as u64);

        let fb = unsafe { physalloc(256).unwrap() };
        self.fb.write(fb as u64);

        let clb = unsafe { physmap(clb_phys, size_of::<HbaCmdHeader>(), MAP_WRITE).unwrap() } as *mut HbaCmdHeader;
        for i in 0..32 {
            let cmdheader = unsafe { &mut *clb.offset(i) };
            let ctba = unsafe { physalloc(size_of::<HbaCmdTable>()).unwrap() };
            cmdheader.ctba.write(ctba as u64);
            cmdheader.prdtl.write(0);
        }
        unsafe { physunmap(clb as usize).unwrap(); }

        self.start();
    }

    pub unsafe fn identify(&mut self, port: usize) -> Option<u64> {
        self.is.write(u32::MAX);

        let dest_phys = physalloc(256).unwrap();
        let dest = physmap(dest_phys, 256, MAP_WRITE).unwrap() as *mut u16;

        if let Some(slot) = self.slot() {
            {
                let clb = unsafe { physmap(self.clb.read() as usize, size_of::<HbaCmdHeader>(), MAP_WRITE).unwrap() } as *mut HbaCmdHeader;
                let cmdheader = &mut *clb.offset(slot as isize);
                cmdheader.cfl.write(((size_of::<FisRegH2D>() / size_of::<u32>()) as u8));
                cmdheader.prdtl.write(1);

                let ctba = unsafe { physmap(cmdheader.ctba.read() as usize, size_of::<HbaCmdTable>(), MAP_WRITE).unwrap() } as *mut HbaCmdTable;
                ptr::write_bytes(ctba as *mut u8, 0, size_of::<HbaCmdTable>());
                let cmdtbl = &mut *(ctba);

                let prdt_entry = &mut cmdtbl.prdt_entry[0];
                prdt_entry.dba.write(dest_phys as u64);
                prdt_entry.dbc.write(512 | 1);

                let cmdfis = &mut *(cmdtbl.cfis.as_ptr() as *mut FisRegH2D);

                cmdfis.fis_type.write(FisType::RegH2D as u8);
                cmdfis.pm.write(1 << 7);
                cmdfis.command.write(ATA_CMD_IDENTIFY);
                cmdfis.device.write(0);
                cmdfis.countl.write(1);
                cmdfis.counth.write(0);

                unsafe { physunmap(ctba as usize).unwrap(); }
                unsafe { physunmap(clb as usize).unwrap(); }
            }

            while self.tfd.readf((ATA_DEV_BUSY | ATA_DEV_DRQ) as u32) {}

            self.ci.writef(1 << slot, true);

            while self.ci.readf(1 << slot) {
                if self.is.readf(HBA_PORT_IS_TFES) {
                    return None;
                }
            }

            if self.is.readf(HBA_PORT_IS_TFES) {
                return None;
            }

            let mut serial = String::new();
            for word in 10..20 {
                let d = *dest.offset(word);
                let a = ((d >> 8) as u8) as char;
                if a != '\0' {
                    serial.push(a);
                }
                let b = (d as u8) as char;
                if b != '\0' {
                    serial.push(b);
                }
            }

            let mut firmware = String::new();
            for word in 23..27 {
                let d = *dest.offset(word);
                let a = ((d >> 8) as u8) as char;
                if a != '\0' {
                    firmware.push(a);
                }
                let b = (d as u8) as char;
                if b != '\0' {
                    firmware.push(b);
                }
            }

            let mut model = String::new();
            for word in 27..47 {
                let d = *dest.offset(word);
                let a = ((d >> 8) as u8) as char;
                if a != '\0' {
                    model.push(a);
                }
                let b = (d as u8) as char;
                if b != '\0' {
                    model.push(b);
                }
            }

            let mut sectors = (*dest.offset(100) as u64) |
                              ((*dest.offset(101) as u64) << 16) |
                              ((*dest.offset(102) as u64) << 32) |
                              ((*dest.offset(103) as u64) << 48);

            let lba_bits = if sectors == 0 {
                sectors = (*dest.offset(60) as u64) | ((*dest.offset(61) as u64) << 16);
                28
            } else {
                48
            };

            println!("   + Port {}: Serial: {} Firmware: {} Model: {} {}-bit LBA Size: {} MB",
                        port, serial.trim(), firmware.trim(), model.trim(), lba_bits, sectors / 2048);

            Some(sectors * 512)
        } else {
            None
        }
    }

    pub fn start(&mut self) {
        while self.cmd.readf(HBA_PORT_CMD_CR) {}

        self.cmd.writef(HBA_PORT_CMD_FRE, true);
        self.cmd.writef(HBA_PORT_CMD_ST, true);
    }

    pub fn stop(&mut self) {
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
                println!("Slot {}", slot);

                let clb = self.clb.read() as usize;
                let cmdheader = unsafe { &mut *(clb as *mut HbaCmdHeader).offset(slot as isize) };

                cmdheader.cfl.write(((size_of::<FisRegH2D>() / size_of::<u32>()) as u8));
                cmdheader.cfl.writef(1 << 6, write);

                cmdheader.prdtl.write(entries);

                let ctba = cmdheader.ctba.read() as usize;
                unsafe { ptr::write_bytes(ctba as *mut u8, 0, size_of::<HbaCmdTable>()) };
                let cmdtbl = unsafe { &mut *(ctba as *mut HbaCmdTable) };

                let prdt_entry = &mut cmdtbl.prdt_entry[0];
                prdt_entry.dba.write(buf as u64);
                prdt_entry.dbc.write(((sectors * 512) as u32) | 1);

                let cmdfis = unsafe { &mut *(cmdtbl.cfis.as_ptr() as *mut FisRegH2D) };

                cmdfis.fis_type.write(FisType::RegH2D as u8);
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

                println!("Busy Wait");
                while self.tfd.readf((ATA_DEV_BUSY | ATA_DEV_DRQ) as u32) {}

                self.ci.writef(1 << slot, true);

                println!("Completion Wait");
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
                println!("No Command Slots");
                Err(Error::new(EIO))
            }
        } else {
            println!("Invalid request");
            Err(Error::new(EIO))
        }
    }

    pub fn ata_dma(&mut self, block: u64, sectors: usize, buf: usize, write: bool) -> Result<usize> {
        println!("AHCI {:X} DMA BLOCK: {:X} SECTORS: {} BUF: {:X} WRITE: {}", (self as *mut HbaPort) as usize, block, sectors, buf, write);

        if sectors > 0 {
            let physical_address = try!(unsafe { virttophys(buf) });

            let mut sector: usize = 0;
            while sectors - sector >= 255 {
                if let Err(err) = self.ata_dma_small(block + sector as u64, 255, physical_address + sector * 512, write) {
                    return Err(err);
                }

                sector += 255;
            }
            if sector < sectors {
                if let Err(err) = self.ata_dma_small(block + sector as u64, sectors - sector, physical_address + sector * 512, write) {
                    return Err(err);
                }
            }

            Ok(sectors * 512)
        } else {
            println!("Invalid request");
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
