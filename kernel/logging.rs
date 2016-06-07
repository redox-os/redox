use collections::borrow::ToOwned;
use collections::String;

use common::time::Duration;

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

#[macro_export]
macro_rules! syslog_debug {
    ($($arg:tt)*) => ({
        $crate::logging::syslog_inner($crate::logging::LogLevel::Debug, format!($($arg)*));
    });
}

#[macro_export]
macro_rules! syslog_info {
    ($($arg:tt)*) => ({
        $crate::logging::syslog_inner($crate::logging::LogLevel::Info, format!($($arg)*));
    });
}

#[macro_export]
macro_rules! syslog_warning {
    ($($arg:tt)*) => ({
        $crate::logging::syslog_inner($crate::logging::LogLevel::Warning, format!($($arg)*));
    });
}

#[macro_export]
macro_rules! syslog_critical {
    ($($arg:tt)*) => ({
        $crate::logging::syslog_inner($crate::logging::LogLevel::Critical, format!($($arg)*));
    });
}

#[macro_export]
macro_rules! syslog_error {
    ($($arg:tt)*) => ({
        $crate::logging::syslog_inner($crate::logging::LogLevel::Error, format!($($arg)*));
    });
}

/// Add `message` to the kernel logs, with a priority level of `level`
pub fn syslog(level: LogLevel, message: &str) {
    syslog_inner(level, message.to_owned());
}

//TODO: Limit log message size
pub fn syslog_inner(level: LogLevel, message: String) {
    let time = Duration::monotonic();
    let logs = unsafe { &mut *::env().logs.get() };
    while logs.len() >= 4096 {
        logs.pop_front();
    }
    logs.push_back((time, level, message));
    //TODO: Print messages that are above priority
}
