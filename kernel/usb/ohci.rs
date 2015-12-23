use alloc::boxed::Box;

use collections::string::ToString;

use core::intrinsics::{volatile_load, volatile_store};
use core::{cmp, mem, ptr};

use scheduler::context::{self, Context};
use common::debug;
use common::event::MouseEvent;
use common::memory::{self, Memory};
use common::time::{self, Duration};

use drivers::pciconfig::PciConfig;
use drivers::pio::*;

use graphics::display::VBEMODEINFO;

use schemes::KScheme;

use sync::Intex;

pub struct Ohci {
    pub base: usize,
    pub irq: u8,
}

impl KScheme for Ohci {
    fn on_irq(&mut self, irq: u8) {
        if irq == self.irq {
            // d("OHCI IRQ\n");
        }
    }

    fn on_poll(&mut self) {
    }
}

impl Ohci {
    pub unsafe fn new(mut pci: PciConfig) -> Box<Self> {
        pci.flag(4, 4, true); // Bus mastering

        let module = box Ohci {
            base: pci.read(0x10) as usize & 0xFFFFFFF0,
            irq: pci.read(0x3C) as u8 & 0xF,
        };

        module.init();

        return module;
    }

    pub unsafe fn init(&self) {
        debug::d("OHCI on: ");
        debug::dh(self.base);
        debug::d(", IRQ: ");
        debug::dbh(self.irq);
        debug::dl();
    }
}
