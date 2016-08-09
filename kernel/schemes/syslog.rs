use fs::{KScheme, Resource};
use alloc::boxed::Box;
use system::error::Result;

/// The kernel log scheme.
pub struct SyslogScheme;

impl KScheme for SyslogScheme {
    /// Returns the name of the scheme
    fn scheme(&self) -> &str {
        "syslog"
    }

    /// Returns a resource. The `url` and `flags` arguments are currently unused.
    fn open(&mut self, _: &str, _: usize) -> Result<Box<Resource>> {
        Ok(Box::new(SyslogResource {
            pos: 0
        }))
    }
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
