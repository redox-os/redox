macro_rules! assume {
    ($cond:expr) => (
        if cfg!(debug) && !$cond {
            panic!(concat!("assertion failed: ", stringify!($cond)))
        } else {
            use core::intrinsics::assume;

            assume($cond);
        }
    );
    ($cond:expr, $($arg:tt)+) => (
        if cfg!(debug) && !$cond {
            panic!($($arg)+)
        } else {
            use core::intrinsics::assume;

            assume($cond);
        }
    );
}
