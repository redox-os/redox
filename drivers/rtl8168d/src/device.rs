use std::{cmp, mem, slice};

use dma::Dma;
use io::{Mmio, Io, ReadOnly, WriteOnly};
use syscall::error::{Error, EACCES, EWOULDBLOCK, Result};
use syscall::scheme::SchemeMut;

#[repr(packed)]
struct Regs {
    mac: [Mmio<u32>; 2],
    mar: Mmio<u64>,
    dtccr: Mmio<u64>,
    _rsv0: Mmio<u64>,
    tnpds: Mmio<u64>,
    thpds: Mmio<u64>,
    _rsv1: [Mmio<u8>; 7],
    cmd: Mmio<u8>,
    tppoll: WriteOnly<Mmio<u8>>,
    _rsv2: [Mmio<u8>; 3],
    imr: Mmio<u16>,
    isr: Mmio<u16>,
    tcr: Mmio<u32>,
    rcr: Mmio<u32>,
    tctr: Mmio<u32>,
    _rsv3: Mmio<u32>,
    cmd_9346: Mmio<u8>,
    config: [Mmio<u8>; 6],
    _rsv4: Mmio<u8>,
    timer_int: Mmio<u32>,
    _rsv5: Mmio<u32>,
    phys_ar: Mmio<u32>,
    _rsv6: Mmio<u64>,
    phys_sts: ReadOnly<Mmio<u8>>,
    _rsv7: [Mmio<u8>; 23],
    wakeup: [Mmio<u64>; 8],
    crc: [Mmio<u16>; 5],
    _rsv8: [Mmio<u8>; 12],
    rms: Mmio<u16>,
    _rsv9: Mmio<u32>,
    c_plus_cr: Mmio<u16>,
    _rsv10: Mmio<u16>,
    rdsar: Mmio<u64>,
    mtps: Mmio<u8>,
    _rsv11: [Mmio<u8>; 19],
}

const OWN: u16 = 1 << 15;
const EOR: u16 = 1 << 14;

#[repr(packed)]
struct Rd {
    length: Mmio<u16>,
    flags: Mmio<u16>,
    vlan: Mmio<u32>,
    buffer: Mmio<u64>
}

#[repr(packed)]
struct Td {
    length: Mmio<u16>,
    flags: Mmio<u16>,
    vlan: Mmio<u32>,
    buffer: Mmio<u64>
}

pub struct Rtl8168 {
    regs: &'static mut Regs,
    irq: u8,
    receive_buffer: [Dma<[u8; 0x1FF8]>; 16],
    receive_ring: Dma<[Rd; 16]>,
    transmit_buffer: [Dma<[u8; 0x1FF8]>; 16],
    transmit_ring: Dma<[Td; 16]>,
    transmit_buffer_h: [Dma<[u8; 0x1FF8]>; 1],
    transmit_ring_h: Dma<[Td; 1]>
}

impl SchemeMut for Rtl8168 {
    fn open(&mut self, _path: &[u8], _flags: usize, uid: u32, _gid: u32) -> Result<usize> {
        if uid == 0 {
            Ok(0)
        } else {
            Err(Error::new(EACCES))
        }
    }

    fn dup(&mut self, id: usize) -> Result<usize> {
        Ok(id)
    }

    fn read(&mut self, _id: usize, buf: &mut [u8]) -> Result<usize> {
        println!("Try Receive {}", buf.len());
        for (rd_i, rd) in self.receive_ring.iter_mut().enumerate() {
            if ! rd.flags.readf(OWN) {
                println!("Receive {}: {}", rd_i, rd.length.read());

                let data = &self.receive_buffer[rd_i as usize][.. rd.length.read() as usize];

                let mut i = 0;
                while i < buf.len() && i < data.len() {
                    buf[i] = data[i];
                    i += 1;
                }

                rd.flags.writef(OWN, true);

                return Ok(i);
            }
        }

        Err(Error::new(EWOULDBLOCK))
    }

    fn write(&mut self, _id: usize, buf: &[u8]) -> Result<usize> {
        println!("Try Transmit {}", buf.len());
        loop {
            for (td_i, td) in self.transmit_ring.iter_mut().enumerate() {
                if ! td.flags.readf(OWN) {
                    println!("Transmit {}: Setup {}", td_i, buf.len());

                    let mut data = &mut self.transmit_buffer[td_i as usize];

                    let mut i = 0;
                    while i < buf.len() && i < data.len() {
                        data[i] = buf[i];
                        i += 1;
                    }

                    td.length.write(cmp::min(buf.len(), i) as u16);

                    td.flags.writef(OWN | 1 << 13 | 1 << 12, true);

                    self.regs.tppoll.writef(1 << 6, true); //Notify of normal priority packet

                    return Ok(i);
                }
            }

            unsafe { asm!("pause" : : : "memory" : "intel", "volatile"); }
        }
    }

    fn fsync(&mut self, _id: usize) -> Result<usize> {
        Ok(0)
    }

    fn close(&mut self, _id: usize) -> Result<usize> {
        Ok(0)
    }
}

impl Rtl8168 {
    pub unsafe fn new(base: usize, irq: u8) -> Result<Self> {
        assert_eq!(mem::size_of::<Regs>(), 256);

        let regs = &mut *(base as *mut Regs);
        assert_eq!(&regs.tnpds as *const _ as usize - base, 0x20);
        assert_eq!(&regs.cmd as *const _ as usize - base, 0x37);
        assert_eq!(&regs.tcr as *const _ as usize - base, 0x40);
        assert_eq!(&regs.rcr as *const _ as usize - base, 0x44);
        assert_eq!(&regs.cmd_9346 as *const _ as usize - base, 0x50);
        assert_eq!(&regs.phys_sts as *const _ as usize - base, 0x6C);
        assert_eq!(&regs.rms as *const _ as usize - base, 0xDA);
        assert_eq!(&regs.rdsar as *const _ as usize - base, 0xE4);
        assert_eq!(&regs.mtps as *const _ as usize - base, 0xEC);

        let mut module = Rtl8168 {
            regs: regs,
            irq: irq,
            receive_buffer: [Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
                            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
                            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
                            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?],
            receive_ring: Dma::zeroed()?,
            transmit_buffer: [Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
                            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
                            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?,
                            Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?, Dma::zeroed()?],
            transmit_ring: Dma::zeroed()?,
            transmit_buffer_h: [Dma::zeroed()?],
            transmit_ring_h: Dma::zeroed()?
        };

        module.init();

        Ok(module)
    }

    pub unsafe fn irq(&mut self) -> u16 {
        // Read and then clear the ISR
        let isr = self.regs.isr.read();
        self.regs.isr.write(isr);
        isr
    }

    pub unsafe fn init(&mut self) {
        println!(" + RTL8168 on: {:X}, IRQ: {}", self.regs as *mut Regs as usize, self.irq);

        let mac_low = self.regs.mac[0].read();
        let mac_high = self.regs.mac[1].read();
        let mac = [mac_low as u8,
                    (mac_low >> 8) as u8,
                    (mac_low >> 16) as u8,
                    (mac_low >> 24) as u8,
                    mac_high as u8,
                    (mac_high >> 8) as u8];
        println!("   - MAC: {:>02X}:{:>02X}:{:>02X}:{:>02X}:{:>02X}:{:>02X}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

        // Reset - this will disable tx and rx, reinitialize FIFOs, and set the system buffer pointer to the initial value
        self.regs.cmd.writef(1 << 4, true);
        while self.regs.cmd.readf(1 << 4) {}

        // Set up rx buffers
        for i in 0..self.receive_ring.len() {
            self.receive_ring[i].flags.writef(OWN, true);
            self.receive_ring[i].length.write(self.receive_buffer[i].len() as u16);
            self.receive_ring[i].buffer.write(self.receive_buffer[i].physical() as u64);
        }
        if let Some(mut rd) = self.receive_ring.last_mut() {
            rd.flags.writef(OWN | EOR, true);
        }

        // Set up normal priority tx buffers
        for i in 0..self.transmit_ring.len() {
            self.transmit_ring[i].buffer.write(self.transmit_buffer[i].physical() as u64);
        }
        if let Some(mut td) = self.transmit_ring.last_mut() {
            td.flags.writef(EOR, true);
        }

        // Set up high priority tx buffers
        for i in 0..self.transmit_ring_h.len() {
            self.transmit_ring_h[i].buffer.write(self.transmit_buffer_h[i].physical() as u64);
        }
        if let Some(mut td) = self.transmit_ring_h.last_mut() {
            td.flags.writef(EOR, true);
        }

        // Unlock config
        self.regs.cmd_9346.write(1 << 7 | 1 << 6);

        // Accept broadcast (bit 3), multicast (bit 2), and unicast (bit 1)
        self.regs.rcr.writef(0xE70F /*TODO: Not permiscuious*/, true);

        // Enable tx (bit 2)
        self.regs.cmd.writef(1 << 2, true);

        // Set TX config
        self.regs.tcr.write(0x03010700);

        // Max RX packet size
        self.regs.rms.write(0x1FF8);

        // Max TX packet size
        self.regs.mtps.write(0x3B);

        // Set tx low priority buffer address
        self.regs.tnpds.write(self.transmit_ring.physical() as u64);

        // Set tx high priority buffer address
        self.regs.thpds.write(self.transmit_ring_h.physical() as u64);

        // Set rx buffer address
        self.regs.rdsar.write(self.receive_ring.physical() as u64);

        // Enable rx (bit 3) and tx (bit 2)
        self.regs.cmd.writef(1 << 3 | 1 << 2, true);

        // Interrupt on tx error (bit 3), tx ok (bit 2), rx error(bit 1), and rx ok (bit 0)
        self.regs.imr.write(1 << 15 | 1 << 14 | 1 << 7 | 1 << 6 | 1 << 4 | 1 << 3 | 1 << 2 | 1 << 1 | 1);

        // Lock config
        self.regs.cmd_9346.write(0);

        println!("   - Ready {:X}", self.regs.phys_sts.read());
    }
}
