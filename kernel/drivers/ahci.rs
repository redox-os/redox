use alloc::boxed::Box;

use schemes::KScheme;

use super::pciconfig::PciConfig;

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
                debugln!("Port {}: {:X}", i, mem.ports[i].ssts);
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
