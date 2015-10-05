use drivers::pio::*;

#[derive(Copy, Clone)]
pub struct PCIConfig {
    bus: u8,
    slot: u8,
    func: u8,
    addr: PIO32,
    data: PIO32,
}

impl PCIConfig {
    pub fn new(bus: u8, slot: u8, func: u8) -> PCIConfig {
        return PCIConfig {
            bus: bus,
            slot: slot,
            func: func,
            addr: PIO32::new(0xCF8),
            data: PIO32::new(0xCFC),
        };
    }

    fn address(&self, offset: u8) -> u32 {
        return 1 << 31 | (self.bus as u32) << 16 | (self.slot as u32) << 11 |
               (self.func as u32) << 8 | (offset as u32 & 0xFC);
    }

    pub unsafe fn read(&mut self, offset: u8) -> u32 {
        let address = self.address(offset);
        self.addr.write(address);
        return self.data.read();
    }

    pub unsafe fn write(&mut self, offset: u8, value: u32) {
        let address = self.address(offset);
        self.addr.write(address);
        self.data.write(value);
    }

    pub unsafe fn flag(&mut self, offset: u8, flag: u32, toggle: bool) {
        let mut value = self.read(offset);
        if toggle {
            value |= flag;
        } else {
            value &= 0xFFFFFFFF - flag;
        }
        self.write(offset, value);
    }

    //TODO: Write functions to get data structures
}
