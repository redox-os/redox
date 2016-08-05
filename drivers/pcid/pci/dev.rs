use super::{PciBus, PciFunc};

pub struct PciDev<'pci> {
    pub bus: &'pci PciBus<'pci>,
    pub num: u8
}

impl<'pci> PciDev<'pci> {
    pub fn funcs(&'pci self) -> PciDevIter<'pci> {
        PciDevIter::new(self)
    }

    pub unsafe fn read(&self, func: u8, offset: u8) -> u32 {
        self.bus.read(self.num, func, offset)
    }
}

pub struct PciDevIter<'pci> {
    dev: &'pci PciDev<'pci>,
    num: u32
}

impl<'pci> PciDevIter<'pci> {
    pub fn new(dev: &'pci PciDev<'pci>) -> Self {
        PciDevIter {
            dev: dev,
            num: 0
        }
    }
}

impl<'pci> Iterator for PciDevIter<'pci> {
    type Item = PciFunc<'pci>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.num < 8 {
            let func = PciFunc {
                dev: self.dev,
                num: self.num as u8
            };
            self.num += 1;
            Some(func)
        } else {
            None
        }
    }
}
