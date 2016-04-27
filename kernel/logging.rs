use collections::borrow::ToOwned;
use collections::String;

#[derive(Copy, Clone)]
pub enum LogLevel {
    Critical,
    Error,
    Warning,
    Info,
    Debug,
}

/// Add message to kernel logs with format
#[macro_export]
macro_rules! syslog {
    ($level:expr, $($arg:tt)*) => ({
        $crate::logging::syslog_inner($level, format!($($arg)*));
    });
}

/// Add `message` to the kernel logs, with a priority level of `level`
pub fn syslog(level: LogLevel, message: &str) {
    syslog_inner(level, message.to_owned());
}

pub fn syslog_inner(level: LogLevel, message: String) {
    let mut logs = ::env().logs.lock();
    logs.push((level, message));
    //TODO: Print messages that are above priority
}
