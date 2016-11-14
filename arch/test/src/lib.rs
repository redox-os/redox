//! Architecture support for testing

pub use std::io;

/// Print to console
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use $crate::io::Write;
        let _ = write!($crate::io::stdout(), $($arg)*);
    });
}

/// Print with new line to console
#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// Create an interrupt function that can safely run rust code
#[macro_export]
macro_rules! interrupt {
    ($name:ident, $func:block) => {
        pub unsafe extern fn $name () {
            unsafe fn inner() {
                $func
            }

            // Call inner rust function
            inner();
        }
    };
}

/// Interrupt instructions
pub mod interrupt;

/// Initialization and main function
pub mod main;

/// Time functions
pub mod time;
