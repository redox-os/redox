use core::{cmp, mem};
use super::Resource;
use system::error::Result;
use system::scheme::Packet;
use arch::context::{Context, context_switch};

/// A supervisor resource.
///
/// Reading from it will simply read the relevant registers to the buffer (see `Packet`).
///
/// Writing will simply left shift EAX by one byte, and then OR it with the byte from the buffer,
/// effectively writing the buffer to the EAX register (truncating the additional bytes).
pub struct SupervisorResource {
    /// The jailed context.
    ctx: *mut Context,
}

impl SupervisorResource {
    /// Create a new supervisor resource, supervising some PID.
    pub unsafe fn new(ctx: *mut Context) -> Result<SupervisorResource> {
        Ok(SupervisorResource {
            ctx: ctx,
        })
    }
}

impl Resource for SupervisorResource {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut _contexts = ::env().contexts.lock();

        let ctx = unsafe { &mut *self.ctx };
        while !ctx.blocked_syscall {
            unsafe { context_switch() };
        }

        let call: Packet = ctx.regs.into();

        for (&a, b) in call.iter().zip(buf.iter_mut()) {
            *b = a;
        }

        ctx.blocked_syscall = false;

        Ok(cmp::min(buf.len(), call.len()))
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut _contexts = ::env().contexts.lock();

        let ctx = unsafe { &mut *self.ctx };

        for &i in buf.iter().take(mem::size_of::<usize>()) {
            ctx.regs.ax <<= 8;
            ctx.regs.ax |= i as usize;
        }

        ctx.blocked = false;

        Ok(cmp::min(mem::size_of::<usize>(), buf.len()))
    }

    // TODO implement seek?
}
