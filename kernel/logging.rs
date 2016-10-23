use core::fmt;

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

/// The maximal total length of a single log message, in bytes.
const MAX_LOG_MESSAGE_SIZE: usize = 512;

pub fn syslog_inner(level: LogLevel, message: fmt::Arguments) {
    let time = Duration::monotonic();

    let (prefix, display) = match level {
        LogLevel::Debug    => ("DEBUG ", false),
        LogLevel::Info     => ("INFO  ", true),
        LogLevel::Warning  => ("WARN  ", true),
        LogLevel::Error    => ("ERROR ", true),
        LogLevel::Critical => ("CRIT  ", true),
    };

    let mut contents = format!("[{}.{:>03}] {}{}\n", time.secs, time.nanos/1000000, prefix, message);
    contents.truncate(MAX_LOG_MESSAGE_SIZE);
    let _ = unsafe { &mut *::env().log.get() }.write(contents.as_bytes());
    if display {
        let _ = ::common::debug::SerialConsole::new().write(contents.as_bytes());
    }
}
