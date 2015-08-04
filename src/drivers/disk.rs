use core::mem::size_of;

use common::debug::*;
use common::memory::*;
use common::pio::*;

//Status port bits
const ATA_SR_BSY: u8 = 0x80;
const ATA_SR_DRDY: u8 = 0x40;
const ATA_SR_DF: u8 = 0x20;
const ATA_SR_DSC: u8 = 0x10;
const ATA_SR_DRQ: u8 = 0x08;
const ATA_SR_CORR: u8 = 0x04;
const ATA_SR_IDX: u8 = 0x02;
const ATA_SR_ERR: u8 = 0x01;

//Error port bits
const ATA_ER_BBK: u8 = 0x80;
const ATA_ER_UNC: u8 = 0x40;
const ATA_ER_MC: u8 = 0x20;
const ATA_ER_IDNF: u8 = 0x10;
const ATA_ER_MCR: u8 = 0x08;
const ATA_ER_ABRT: u8 = 0x04;
const ATA_ER_TK0NF: u8 = 0x02;
const ATA_ER_AMNF: u8 = 0x01;

//Commands
const ATA_CMD_READ_PIO: u8 = 0x20;
const ATA_CMD_READ_PIO_EXT: u8 = 0x24;
const ATA_CMD_READ_DMA: u8 = 0xC8;
const ATA_CMD_READ_DMA_EXT: u8 = 0x25;
const ATA_CMD_WRITE_PIO: u8 = 0x30;
const ATA_CMD_WRITE_PIO_EXT: u8 = 0x34;
const ATA_CMD_WRITE_DMA: u8 = 0xCA;
const ATA_CMD_WRITE_DMA_EXT: u8 = 0x35;
const ATA_CMD_CACHE_FLUSH: u8 = 0xE7;
const ATA_CMD_CACHE_FLUSH_EXT: u8 = 0xEA;
const ATA_CMD_PACKET: u8 = 0xA0;
const ATA_CMD_IDENTIFY_PACKET: u8 = 0xA1;
const ATA_CMD_IDENTIFY: u8 = 0xEC;

//Identification
const ATA_IDENT_DEVICETYPE: u8 = 0;
const ATA_IDENT_CYLINDERS: u8 = 2;
const ATA_IDENT_HEADS: u8 = 6;
const ATA_IDENT_SECTORS: u8 = 12;
const ATA_IDENT_SERIAL: u8 = 20;
const ATA_IDENT_MODEL: u8 = 54;
const ATA_IDENT_CAPABILITIES: u8 = 98;
const ATA_IDENT_FIELDVALID: u8 = 106;
const ATA_IDENT_MAX_LBA: u8 = 120;
const ATA_IDENT_COMMANDSETS: u8 = 164;
const ATA_IDENT_MAX_LBA_EXT: u8 = 200;

//Selection
const ATA_MASTER: u8 = 0x00;
const ATA_SLAVE: u8 = 0x01;


//Types
const IDE_ATA: u8 = 0x00;
const IDE_ATAPI: u8 = 0x01;

//Registers
const ATA_REG_DATA: u16 = 0x00;
const ATA_REG_ERROR: u16 = 0x01;
const ATA_REG_FEATURES: u16 = 0x01;
const ATA_REG_SECCOUNT0: u16 = 0x02;
const ATA_REG_LBA0: u16 = 0x03;
const ATA_REG_LBA1: u16 = 0x04;
const ATA_REG_LBA2: u16 = 0x05;
const ATA_REG_HDDEVSEL: u16 = 0x06;
const ATA_REG_COMMAND: u16 = 0x07;
const ATA_REG_STATUS: u16 = 0x07;
const ATA_REG_SECCOUNT1: u16 = 0x08;
const ATA_REG_LBA3: u16 = 0x09;
const ATA_REG_LBA4: u16 = 0x0A;
const ATA_REG_LBA5: u16 = 0x0B;
const ATA_REG_CONTROL: u16 = 0x0C;
const ATA_REG_ALTSTATUS: u16 = 0x0C;
const ATA_REG_DEVADDRESS: u16 = 0x0D;

pub struct PRDTE {
    pub ptr: u32,
    pub size: u16,
    pub reserved: u16
}

pub struct Disk{
    base: u16,
    ctrl: u16
}

impl Disk {
    pub fn new() -> Disk{
        Disk{
            base: 0x1F0,
            ctrl: 0x3F4
        }
    }

    unsafe fn ide_read(&self, reg: u16) -> u8{
        let ret;
        if reg < 0x08 {
            ret = inb(self.base + reg - 0x00);
        } else if reg < 0x0C {
            ret = inb(self.base + reg - 0x06);
        } else if reg < 0x0E {
            ret = inb(self.ctrl + reg - 0x0A);
        }else{
            ret = 0;
        }
        ret
    }

    unsafe fn ide_write(&self, reg: u16, data: u8){
        if reg < 0x08 {
            outb(self.base + reg - 0x00, data);
        } else if reg < 0x0C {
            outb(self.base + reg - 0x06, data);
        } else if reg < 0x0E {
            outb(self.ctrl + reg - 0x0A, data);
        }
    }

    unsafe fn ide_poll(&self, check_error: bool) -> u8{
        self.ide_read(ATA_REG_ALTSTATUS);
        self.ide_read(ATA_REG_ALTSTATUS);
        self.ide_read(ATA_REG_ALTSTATUS);
        self.ide_read(ATA_REG_ALTSTATUS);

        while self.ide_read(ATA_REG_STATUS) & ATA_SR_BSY == ATA_SR_BSY {

        }

        if check_error {
            let state = self.ide_read(ATA_REG_STATUS);
            if state & ATA_SR_ERR == ATA_SR_ERR {
                return 2;
            }
            if state & ATA_SR_DF == ATA_SR_DF {
                return 1;
            }
            if !(state & ATA_SR_DRQ == ATA_SR_DRQ) {
                return 3;
            }
        }

        return 0;
    }

    //TODO: Make sure count is not zero!
    pub unsafe fn read(&self, lba: u64, count: u16, destination: usize) -> u8{
        if destination > 0 {
            while self.ide_read(ATA_REG_STATUS) & ATA_SR_BSY == ATA_SR_BSY {

            }

            self.ide_write(ATA_REG_HDDEVSEL, 0x40);

            self.ide_write(ATA_REG_SECCOUNT1, ((count >> 8) & 0xFF) as u8);
            self.ide_write(ATA_REG_LBA3, ((lba >> 24) & 0xFF) as u8);
            self.ide_write(ATA_REG_LBA4, ((lba >> 32) & 0xFF) as u8);
            self.ide_write(ATA_REG_LBA5, ((lba >> 40) & 0xFF) as u8);

            self.ide_write(ATA_REG_SECCOUNT0, ((count >> 0) & 0xFF) as u8);
            self.ide_write(ATA_REG_LBA0, (lba & 0xFF) as u8);
            self.ide_write(ATA_REG_LBA1, ((lba >> 8) & 0xFF) as u8);
            self.ide_write(ATA_REG_LBA2, ((lba >> 16) & 0xFF) as u8);
            self.ide_write(ATA_REG_COMMAND, ATA_CMD_READ_PIO_EXT);

            for sector in 0..count as usize {
                let err = self.ide_poll(true);
                if err > 0{
                    return err;
                }

                for word in 0..256 {
                    let data = inw(self.base + ATA_REG_DATA);
                    *((destination + sector*512 + word*2) as *mut u16) = data;
                }
            }
        }

        return 0;
    }

    pub unsafe fn read_dma(&self, lba: u64, count: u16, destination: usize, busmaster: u16) -> u8{
        if destination > 0 {
            //Allocate PDTR
            let mut prdt = ind(busmaster + 4) as usize;

            if prdt == 0 {
                prdt = alloc(size_of::<PRDTE>());
                *(prdt as *mut PRDTE) = PRDTE {
                    ptr: destination as u32,
                    size: count * 512,
                    reserved: 0x8000
                };

                outd(busmaster + 4, prdt as u32);

                //Set read bit
                outb(busmaster, 8);

                //Clear interrupt, error bit
                outb(busmaster + 2, 0);

                //DMA Transfer Command
                while self.ide_read(ATA_REG_STATUS) & ATA_SR_BSY == ATA_SR_BSY {

                }

                self.ide_write(ATA_REG_HDDEVSEL, 0x40);

                self.ide_write(ATA_REG_SECCOUNT1, ((count >> 8) & 0xFF) as u8);
                self.ide_write(ATA_REG_LBA3, ((lba >> 24) & 0xFF) as u8);
                self.ide_write(ATA_REG_LBA4, ((lba >> 32) & 0xFF) as u8);
                self.ide_write(ATA_REG_LBA5, ((lba >> 40) & 0xFF) as u8);

                self.ide_write(ATA_REG_SECCOUNT0, ((count >> 0) & 0xFF) as u8);
                self.ide_write(ATA_REG_LBA0, (lba & 0xFF) as u8);
                self.ide_write(ATA_REG_LBA1, ((lba >> 8) & 0xFF) as u8);
                self.ide_write(ATA_REG_LBA2, ((lba >> 16) & 0xFF) as u8);
                self.ide_write(ATA_REG_COMMAND, ATA_CMD_READ_DMA_EXT);

                //Engage bus mastering
                outb(busmaster, inb(busmaster) | 1);
            }else{
                d("Operation Running!\n");
            }
        }

        return 0;
    }

    //TODO: Fix and make sure count is not zero!
    pub unsafe fn write(&self, lba: u64, count: u16, source: usize) -> u8{
        if source > 0 {
            while self.ide_read(ATA_REG_STATUS) & ATA_SR_BSY == ATA_SR_BSY {

            }

            self.ide_write(ATA_REG_HDDEVSEL, 0x40);

            self.ide_write(ATA_REG_SECCOUNT1, ((count >> 8) & 0xFF) as u8);
            self.ide_write(ATA_REG_LBA3, ((lba >> 24) & 0xFF) as u8);
            self.ide_write(ATA_REG_LBA4, ((lba >> 32) & 0xFF) as u8);
            self.ide_write(ATA_REG_LBA5, ((lba >> 40) & 0xFF) as u8);

            self.ide_write(ATA_REG_SECCOUNT0, ((count >> 0) & 0xFF) as u8);
            self.ide_write(ATA_REG_LBA0, ((lba >> 0) & 0xFF) as u8);
            self.ide_write(ATA_REG_LBA1, ((lba >> 8) & 0xFF) as u8);
            self.ide_write(ATA_REG_LBA2, ((lba >> 16) & 0xFF) as u8);

            self.ide_write(ATA_REG_COMMAND, ATA_CMD_WRITE_PIO_EXT);

            for sector in 0..count as usize {
                let err = self.ide_poll(true);
                if err > 0{
                    return err;
                }

                for word in 0..256 {
                    outw(self.base + ATA_REG_DATA, *((source + sector*512 + word*2) as *const u16));
                }

                self.ide_write(ATA_REG_COMMAND, ATA_CMD_CACHE_FLUSH_EXT);
                self.ide_poll(false);
            }
        }

        return 0;
    }
}
