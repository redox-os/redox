use fs::Resource;
use alloc::boxed::Box;
use system::error::Result;

pub fn resource() -> Result<Box<Resource>> {
    Ok(Box::new(SyslogResource {
        pos: 0
    }))
}

/// The kernel log resource.
pub struct SyslogResource {
    pos: usize
}

impl Resource for SyslogResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(Box::new(SyslogResource {
            pos: self.pos
        }))
    }

    /// Fills `buf` with the kernel log. Each message is prefixed by its log level:
    /// - `CRIT`
    /// - `ERROR`
    /// - `WARN`
    /// - `INFO`
    /// - `DEBUG`
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let log = unsafe { & *::env().log.get() };
        let count = log.read_at(self.pos, buf);
        self.pos += count;
        Ok(count)
    }
}
