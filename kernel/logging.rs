use collections::borrow::ToOwned;

#[derive(Copy, Clone)]
pub enum LogLevel {
    Critical,
    Error,
    Warning,
    Info,
    Debug,
}

/// Add `message` to the kernel logs, with a priority level of `level`
pub fn klog(level: LogLevel, message: &str) {
    let mut logs = ::env().logs.lock();
    logs.push((level, message.to_owned()));
}
