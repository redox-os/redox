use drivers::pio::*;

pub struct PCIConfig {
    bus: u8,
    slot: u8,
    func: u8,
    offset: u8,
    addr: PIO32,
    data: PIO32
}

impl PCIConfig {
    pub fn new(bus: u8, slot: u8, func: u8, offset: u8) -> PCIConfig {
        return PCIConfig {
            bus: bus,
            slot: slot,
            func: func,
            offset: offset,
            addr: PIO32::new(0xCF8),
            data: PIO32::new(0xCFC)
        };
    }

    fn address(&self) -> u32 {
        return 1 << 31
            | (self.bus as u32 & 255) << 16
            | (self.slot as u32 & 31) << 11
            | (self.function as u32 & 8) << 8
            | (offset as u32 & 0xFC);
    }

    pub unsafe fn read(&mut self) -> u32 {
        self.addr.write(self.address());
        return self.data.read();
    }

    pub unsafe fn write(&mut self, value: u32) {
        self.addr.write(self.address());
        self.data.write(value);
    }

    //TODO: Write functions to get data structures
}
