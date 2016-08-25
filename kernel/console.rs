use ransid::Console;
use spin::{Once, Mutex, MutexGuard};

/// Console
static CONSOLE: Once<Mutex<Console>> = Once::new();

/// Initialize console, called if needed
fn init_console() -> Mutex<Console> {
    Mutex::new(Console::new(0, 0))
}

/// Get the global console
pub fn console() -> MutexGuard<'static, Console> {
    CONSOLE.call_once(init_console).lock()
}
