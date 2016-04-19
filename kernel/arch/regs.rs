pub use self::arch::*;

use system::scheme::Packet;

#[cfg(target_arch = "x86")]
#[path="x86/regs.rs"]
mod arch;

#[cfg(target_arch = "x86_64")]
#[path="x86_64/regs.rs"]
mod arch;

impl Into<Packet> for Regs {
    fn into(self) -> Packet {
        Packet {
            id: 0, // TODO
            a: self.ax,
            b: self.bx,
            c: self.cx,
            d: self.dx,
        }
    }
}
