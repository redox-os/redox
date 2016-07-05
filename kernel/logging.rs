use core::fmt::{self, Write};

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
        $crate::logging::syslog_inner($level, format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! syslog_debug {
    ($($arg:tt)*) => ({
        $crate::logging::syslog_inner($crate::logging::LogLevel::Debug, format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! syslog_info {
    ($($arg:tt)*) => ({
        $crate::logging::syslog_inner($crate::logging::LogLevel::Info, format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! syslog_warning {
    ($($arg:tt)*) => ({
        $crate::logging::syslog_inner($crate::logging::LogLevel::Warning, format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! syslog_critical {
    ($($arg:tt)*) => ({
        $crate::logging::syslog_inner($crate::logging::LogLevel::Critical, format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! syslog_error {
    ($($arg:tt)*) => ({
        $crate::logging::syslog_inner($crate::logging::LogLevel::Error, format_args!($($arg)*));
    });
}

/// Add `message` to the kernel logs, with a priority level of `level`
pub fn syslog(level: LogLevel, message: &str) {
    syslog_inner(level, format_args!("{}", message));
}

//TODO: Limit log message size
pub fn syslog_inner(level: LogLevel, message: fmt::Arguments) {
    let time = Duration::monotonic();

    let prefix: &str = match level {
        LogLevel::Debug    => "DEBUG ",
        LogLevel::Info     => "INFO  ",
        LogLevel::Warning  => "WARN  ",
        LogLevel::Error    => "ERROR ",
        LogLevel::Critical => "CRIT  ",
    };

    let _ = write!(unsafe { &mut *::env().log.get() }, "[{}.{:>03}] {}{}\n", time.secs, time.nanos/1000000, prefix, message);
    let _ = write!(::common::debug::SerialConsole::new(), "[{}.{:>03}] {}{}\n", time.secs, time.nanos/1000000, prefix, message);
}
