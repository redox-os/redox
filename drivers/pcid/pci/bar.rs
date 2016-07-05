#[derive(Debug)]
pub enum PciBar {
    None,
    Memory(u32),
    Port(u16)
}

impl From<u32> for PciBar {
    fn from(bar: u32) -> PciBar {
        if bar & 0xFFFFFFFC == 0 {
            PciBar::None
        } else if bar & 1 == 0 {
            PciBar::Memory(bar & 0xFFFFFFF0)
        } else {
            PciBar::Port((bar & 0xFFFC) as u16)
        }
    }
}
