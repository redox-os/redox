use collections::string::String;

use system::error::Result;

pub mod ahci;
pub mod ide;

pub trait Disk {
    fn name(&self) -> String;
    fn on_irq(&mut self, irq: u8);
    fn size(&self) -> u64;
    fn read(&mut self, block: u64, buffer: &mut [u8]) -> Result<usize>;
    fn write(&mut self, block: u64, buffer: &[u8]) -> Result<usize>;
}
