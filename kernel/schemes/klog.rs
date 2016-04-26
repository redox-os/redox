use fs::{KScheme, Resource, Url};
use fs::resource::ResourceSeek;
use collections::string::String;
use alloc::boxed::Box;
use system::error::Result;
use logging::LogLevel;

/// The kernel log scheme.
pub struct KlogScheme;

impl KScheme for KlogScheme {
    /// Returns the name of the scheme: "klog"
    fn scheme(&self) -> &str {
        "klog"
    }

    /// Returns a resource. The `url` and `flags` arguments are currently unused.
    fn open(&mut self, _: Url, _: usize) -> Result<Box<Resource>> {
        Ok(Box::new(KlogResource {
            pos: 0,
        }))
    }

    /// Clears the logs.
    fn unlink(&mut self, _: Url) -> Result<()> {
        let mut logs = ::env().logs.lock();
        logs.clear();
        Ok(())
    }
}

/// The kernel log resource.
pub struct KlogResource {
    pos: usize,
}

impl KlogResource {
    fn get_log_str(&self) -> String {
        let ref mut logs = *::env().logs.lock();
        let mut string = String::new();
        for &mut (ref level, ref message) in logs {
            let prefix: &str = match *level {
                LogLevel::Debug    => "DEBUG ",
                LogLevel::Info     => "INFO  ",
                LogLevel::Warning  => "WARN  ",
                LogLevel::Error    => "ERROR ",
                LogLevel::Critical => "CRIT  ",
            };
            string.push_str(prefix);
            string.push_str(message);
            string.push('\n');
        }
        string
    }
}

impl Resource for KlogResource {
    fn dup(&self) -> Result<Box<Resource>> {
        Ok(Box::new(KlogResource {
            pos: self.pos,
        }))
    }

    /// Fills `buf` with the kernel log. Each message is prefixed by its log level:
    /// - `CRIT`
    /// - `ERROR`
    /// - `WARN`
    /// - `INFO`
    /// - `DEBUG`
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;
        let logs = self.get_log_str();
        while i < buf.len() && self.pos < logs.bytes().count() {
            match logs.bytes().nth(self.pos) {
                Some(c) => buf[i] = c,
                None => ()
            }
            i += 1;
            self.pos += 1;
        }
        Ok(i)
    }

    fn seek(&mut self, pos: ResourceSeek) -> Result<usize> {
        match pos {
            ResourceSeek::Start(offset) => self.pos = offset as usize,
            ResourceSeek::Current(offset) => self.pos += offset as usize,
            ResourceSeek::End(offset) => {
                let logs = self.get_log_str();
                self.pos = (logs.bytes().count() as isize + offset) as usize;
            }
        }
        Ok(self.pos)
    }
}
