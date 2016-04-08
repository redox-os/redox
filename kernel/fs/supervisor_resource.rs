use core::{cmp, mem};
use super::Resource;
use system::error::Result;
use system::scheme::Packet;

/// A supervisor resource.
///
/// Reading from it will simply read the relevant registers to the buffer (see `Packet`).
///
/// Writing will simply left shift EAX by one byte, and then OR it with the byte from the buffer,
/// effectively writing the buffer to the EAX register (truncating the additional bytes).
pub struct SupervisorResource {
    pid: usize,
}

impl SupervisorResource {
    /// Create a new supervisor resource, supervising some PID.
    pub fn new(pid: usize) -> SupervisorResource {
        SupervisorResource {
            pid: pid,
        }
    }
}

impl Resource for SupervisorResource {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut contexts = ::env().contexts.lock();
        let ctx = try!(contexts.get_mut(self.pid));
        if !ctx.blocked_syscall {
            return Ok(0);
        }

        let call: Packet = ctx.regs.into();

        for (&a, b) in call.iter().zip(buf.iter_mut()) {
            *b = a;
        }

        ctx.blocked_syscall = false;

        Ok(cmp::min(buf.len(), call.len()))
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut contexts = ::env().contexts.lock();
        let ctx = try!(contexts.get_mut(self.pid));

        for &i in buf.iter().take(mem::size_of::<usize>()) {
            ctx.regs.ax <<= 8;
            ctx.regs.ax |= i as usize;
        }

        ctx.blocked = false;

        Ok(cmp::min(mem::size_of::<usize>(), buf.len()))
    }

    // TODO implement seek?
}
