pub mod common;
pub mod ethernet;
pub mod intel8254x;
pub mod ipv4;
pub mod ipv6;
pub mod rtl8139;
pub mod scheme;
pub mod schemes;

use collections::String;

use system::error::Result;

pub trait Nic {
    fn name(&self) -> String;
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize>;
    fn write(&mut self, buffer: &[u8]) -> Result<usize>;
}
