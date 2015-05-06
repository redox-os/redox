use common::debug::*;
use common::memory::*;
use common::pio::*;

/* Networking { */

const CTRL: u32 = 0x00;
    const CTRL_LRST: u32 = 1 << 3;
    const CTRL_ASDE: u32 = 1 << 5;
    const CTRL_SLU: u32 = 1 << 6;
    const CTRL_ILOS: u32 = 1 << 7;
    const CTRL_VME: u32 = 1 << 30;
    const CTRL_PHY_RST: u32 = 1 << 31;

const STATUS: u32 = 0x08;

const FCAL: u32 = 0x28;
const FCAH: u32 = 0x2C;
const FCT: u32 = 0x30;
const FCTTV: u32 = 0x170;

const IMS: u32 = 0xD0;
    const IMS_LSC: u32 = 1 << 2;
    const IMS_RXSEQ: u32 = 1 << 3;
    const IMS_RXDMT: u32 = 1 << 4;
    const IMS_RX: u32 = 1 << 6;
    const IMS_RXT: u32 = 1 << 7;


const RCTL: u32 = 0x100;
    const RCTL_EN: u32 = 1 << 1;
    const RCTL_LPE: u32 = 1 << 5;
    const RCTL_LBM: u32 = 1 << 6 | 1 << 7;
    const RCTL_BAM: u32 = 1 << 15;
    const RCTL_BSIZE: u32 = 1 << 16 | 1 << 17;
    const RCTL_BSEX: u32 = 1 << 25;
    const RCTL_SECRC: u32 = 1 << 26;

const RDBAL: u32 = 0x2800;
const RDBAH: u32 = 0x2804;
const RDLEN: u32 = 0x2808;
const RDH: u32 = 0x2810;
const RDT: u32 = 0x2818;

const RAL0: u32 = 0x5400;
const RAH0: u32 = 0x5404;


pub struct Intel8254x {
    base: usize,
    memory_mapped: bool
}

impl Intel8254x {
    pub unsafe fn read(&self, register: u32) -> u32 {
        let data;

        if self.memory_mapped {
            data = *((self.base + register as usize) as *mut u32);
        }else{
            outl(self.base as u16, register);
            data = inl((self.base + 4) as u16);
        }


        d("Read ");
        dh(register as usize);
        d(", result ");
        dh(data as usize);
        dl();

        return data;
    }

    pub unsafe fn write(&self, register: u32, data: u32){
        let result;
        if self.memory_mapped {
            *((self.base + register as usize) as *mut u32) = data;
            result = *((self.base + register as usize) as *mut u32);
        }else{
            outl(self.base as u16, register);
            outl((self.base + 4) as u16, data);
            result = inl((self.base + 4) as u16);
        }

        d("Set ");
        dh(register as usize);
        d(" to ");
        dh(data as usize);
        d(", result ");
        dh(result as usize);
        dl();
    }

    pub unsafe fn flag(&self, register: u32, flag: u32, value: bool){
        if value {
            self.write(register, self.read(register) | flag);
        }else{
            self.write(register, self.read(register) & (0xFFFFFFFF - flag));
        }
    }

    pub unsafe fn handle(&self){
        d("Intel 8254x handle\n");
    }

    pub unsafe fn init(&self){
        d("Intel 8254x on: ");
        dh(self.base);
        if self.memory_mapped {
            d(" memory mapped");
        }else{
            d(" port mapped");
        }
        dl();

        self.read(CTRL);
        self.read(STATUS);
        self.read(IMS);

        //Enable auto negotiate, link, clear reset, do not Invert Loss-Of Signal
        self.flag(CTRL, CTRL_ASDE | CTRL_SLU, true);
        self.flag(CTRL, CTRL_LRST, false);
        self.flag(CTRL, CTRL_PHY_RST, false);
        self.flag(CTRL, CTRL_ILOS, false);

        //No flow control
        self.write(FCAH, 0);
        self.write(FCAL, 0);
        self.write(FCT, 0);
        self.write(FCTTV, 0);

        //Do not use VLANs
        self.flag(CTRL, CTRL_VME, false);

        // TODO: Clear statistical counters

        self.write(RAL0, 0x20202020);
        self.write(RAH0, 0x2020);
        /*
        MTA => 0;
        */
        self.write(IMS, IMS_RXT | IMS_RX | IMS_RXDMT | IMS_RXSEQ | IMS_LSC);

        //Receive Buffer
        let receive_ring_length = 4096;
        let receive_ring = alloc(receive_ring_length * 16);
        for i in 0..receive_ring_length {
            let receive_buffer = alloc(4096);
            *((receive_ring + i * 16) as *mut u64) = receive_buffer as u64;
        }

        self.write(RDBAH, 0);
        self.write(RDBAL, receive_ring as u32);
        self.write(RDLEN, (receive_ring_length * 16) as u32);
        self.write(RDH, 0);
        self.write(RDT, (receive_ring_length * 16) as u32);

        self.flag(RCTL, RCTL_EN, true);
        self.flag(RCTL, RCTL_LPE, true);
        self.flag(RCTL, RCTL_LBM, false);
        /* RCTL.RDMTS = Minimum threshold size ??? */
        /* RCTL.MO = Multicast offset */
        self.flag(RCTL, RCTL_BAM, true);
        self.flag(RCTL, RCTL_BSIZE, true);
        self.flag(RCTL, RCTL_BSEX, true);
        self.flag(RCTL, RCTL_SECRC, true);

        self.write(IMS, IMS_RXT | IMS_RX | IMS_RXDMT | IMS_RXSEQ | IMS_LSC);

        /*
        self.flag(TCTL, TCTL_EN, true);
        self.flag(TCTL, TCTL_PSP, true);
        */
        /* TCTL.CT = Collition threshold */
        /* TCTL.COLD = Collision distance */
        /* TIPG Packet Gap */
        /* TODO ... */

        self.read(CTRL);
        self.read(STATUS);
        self.read(IMS);
    }
}

pub struct RTL8139 {
    base: usize,
    memory_mapped: bool,
    receive_buffer: usize
}

impl RTL8139 {
    pub unsafe fn handle(&self){
        d("RTL8139 handle\n");

        let base = self.base as u16;


        let mut capr = (inw(base + 0x38) + 16) as usize;
        let cbr = inw(base + 0x3A) as usize;

        d("CAPR: ");
        dh(capr);
        dl();

        d("CBR: ");
        dh(cbr);
        dl();

        d("Packet len: ");
        let packet_len = *((self.receive_buffer + 2) as *const u16) as usize;
        dh(packet_len);
        dl();

        for i in capr..cbr {
            let data = *((self.receive_buffer + i*4) as *const u8);
            dbh(data);
            if i % 40 == 39 {
                dl();
            }else if i % 4 == 3{
                d(" ");
            }
        }
        dl();

        capr = capr + packet_len + 4;
        capr = (capr + 3) & (0xFFFFFFFF - 3);
        if capr >= 8192 {
            capr -= 8192
        }

        outw(base + 0x38, (capr as u16) - 16);
        outw(base + 0x3E, 0x0001);
    }

    pub unsafe fn init(&self){
        d("RTL8139 on: ");
        dh(self.base);
        if self.memory_mapped {
            d(" memory mapped");
        }else{
            d(" port mapped");
        }
        dl();

        let base = self.base as u16;

        outb(base + 0x52, 0x00);

        outb(base + 0x37, 0x10);
        while inb(base + 0x37) & 0x10 != 0 {
        }

        outl(base + 0x30, self.receive_buffer as u32);
        outw(base + 0x38, 0);
        outw(base + 0x3A, 0);

        outw(base + 0x3C, 0x0005);

        outl(base + 0x44, 0xf | (1 << 7));

        outb(base + 0x37, 0x0C);
    }
}

/* } Networking */

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

unsafe fn pci_read(bus: usize, slot: usize, function: usize, offset: usize) -> usize{
    outl(CONFIG_ADDRESS, ((1 << 31) | (bus << 16) | (slot << 11) | (function << 8) | (offset & 0xfc)) as u32);
    return inl(CONFIG_DATA) as usize;
}

unsafe fn pci_write(bus: usize, slot: usize, function: usize, offset: usize, data: usize){
    outl(CONFIG_ADDRESS, ((1 << 31) | (bus << 16) | (slot << 11) | (function << 8) | (offset & 0xfc)) as u32);
    outl(CONFIG_DATA, data as u32);
}

pub unsafe fn pci_handle(irq: u8){
    d("PCI Handle ");
    dh(irq as usize);
    dl();

    for device in 0..32 {
        let data = pci_read(0, device, 0, 0);

        if (data & 0xFFFF) != 0xFFFF {
            if irq == pci_read(0, device, 0, 0x3C) as u8 & 0xF {
                if data == 0x100E8086 {
                    let base = pci_read(0, device, 0, 0x10);
                    let device = Intel8254x {
                        base: base & (0xFFFFFFFF - 1),
                        memory_mapped: base & 1 == 0
                    };
                    device.handle();
                } else if data == 0x813910EC {
                    let base = pci_read(0, device, 0, 0x10);
                    let device = RTL8139 {
                        base: base & (0xFFFFFFFF - 1),
                        memory_mapped: base & 1 == 0,
                        receive_buffer: 0x2A0000
                    };
                    device.handle();
                }
            }
        }
    }
}

pub unsafe fn pci_test(){
    d("PCI\n");

    for device in 0..32 {
        let data = pci_read(0, device, 0, 0);

        if (data & 0xFFFF) != 0xFFFF {
            d("Device ");
            dd(device);
            d(": ");
            dh(data);
            d(", ");
            dh(pci_read(0, device, 0, 8));
            dl();

            for i in 0..6 {
                d("    ");
                dd(i);
                d(": ");
                dh(pci_read(0, device, 0, i*4 + 0x10));
                dl();
            }

            if data == 0x100E8086 {
                let base = pci_read(0, device, 0, 0x10);
                let device = Intel8254x {
                    base: base & (0xFFFFFFFF - 1),
                    memory_mapped: base & 1 == 0
                };
                device.init();
            } else if data == 0x813910EC {
                pci_write(0, device, 0, 0x04, pci_read(0, device, 0, 0x04) | (1 << 2));

                d("IRQ ");
                dh(pci_read(0, device, 0, 0x3C) & 0xF + 0x20);
                dl();

                let base = pci_read(0, device, 0, 0x10);
                let device = RTL8139 {
                    base: base & (0xFFFFFFFF - 1),
                    memory_mapped: base & 1 == 0,
                    receive_buffer: 0x2A0000
                };
                device.init();
            }

            dl();
        }
    }
}