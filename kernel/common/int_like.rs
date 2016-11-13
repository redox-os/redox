//! Helpers used to define types that are backed by integers (typically `usize`),
//! without compromising safety.
//!
//! # Example
//!
//! ```
//! /// Define an opaque type `Pid` backed by a `usize`.
//! int_like!(Pid, usize);
//!
//! const ZERO: Pid = Pid::from(0);
//! ```
//!
//! # Example
//!
//! ```
//! /// Define opaque types `Pid` and `AtomicPid`, backed respectively by a `usize`
//! /// and a `AtomicUsize`.
//!
//! int_like!(Pid, AtomicPid, usize, AtomicUsize);
//!
//! const ZERO: Pid = Pid::from(0);
//! let ATOMIC_PID: AtomicPid = AtomicPid::default();
//! ```

#[macro_export]
macro_rules! int_like {
    ($new_type_name:ident, $backing_type: ident) => {
        #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
        pub struct $new_type_name($backing_type);

        impl $new_type_name {
            pub const fn into(self) -> $backing_type {
                self.0
            }
            pub const fn from(x: $backing_type) -> Self {
                $new_type_name(x)
            }
        }
    };

    ($new_type_name:ident, $new_atomic_type_name: ident, $backing_type:ident, $backing_atomic_type:ident) => {
        int_like!($new_type_name, $backing_type);

        /// A mutable holder for T that can safely be shared among threads.
        /// Runtime equivalent to using `AtomicUsize`, just type-safer.
        pub struct $new_atomic_type_name {
            container: $backing_atomic_type,
        }

        impl $new_atomic_type_name {
            pub const fn new(x: $new_type_name) -> Self {
                $new_atomic_type_name {
                    container: $backing_atomic_type::new(x.into())
                }
            }
            pub const fn default() -> Self {
                Self::new($new_type_name::from(0))
            }
            pub fn load(&self, order: ::core::sync::atomic::Ordering) -> $new_type_name {
                $new_type_name::from(self.container.load(order))
            }
            pub fn store(&self, val: $new_type_name, order: ::core::sync::atomic::Ordering) {
                self.container.store(val.into(), order)
            }
            #[allow(dead_code)]
            pub fn swap(&self, val: $new_type_name, order: ::core::sync::atomic::Ordering) -> $new_type_name {
                $new_type_name::from(self.container.swap(val.into(), order))
            }
            #[allow(dead_code)]
            pub fn compare_and_swap(&self, current: $new_type_name, new: $new_type_name, order: ::core::sync::atomic::Ordering) -> $new_type_name {
                $new_type_name::from(self.container.compare_and_swap(current.into(), new.into(), order))
            }
            #[allow(dead_code)]
            pub fn compare_exchange(&self, current: $new_type_name, new: $new_type_name, success: ::core::sync::atomic::Ordering, failure: ::core::sync::atomic::Ordering) -> ::core::result::Result<$new_type_name, $new_type_name> {
                match self.container.compare_exchange(current.into(), new.into(), success, failure) {
                    Ok(result) => Ok($new_type_name::from(result)),
                    Err(result) => Err($new_type_name::from(result))
                }
            }
            #[allow(dead_code)]
            pub fn compare_exchange_weak(&self, current: $new_type_name, new: $new_type_name, success: ::core::sync::atomic::Ordering, failure: ::core::sync::atomic::Ordering) -> ::core::result::Result<$new_type_name, $new_type_name> {
                match self.container.compare_exchange_weak(current.into(), new.into(), success, failure) {
                    Ok(result) => Ok($new_type_name::from(result)),
                    Err(result) => Err($new_type_name::from(result))
                }
            }
        }
    }
}

#[cfg(test)]
fn test() {
    use core::mem::size_of;
    use ::core::sync::atomic::AtomicUsize;

    // Generate type `usize_like`.
    int_like!(UsizeLike, usize);
    const ZERO: UsizeLike = UsizeLike::from(0);
    assert_eq!(size_of::<UsizeLike>(), size_of::<usize>());


    // Generate types `usize_like` and `AtomicUsize`.
    int_like!(UsizeLike2, AtomicUsizeLike, usize, AtomicUsize);
    assert_eq!(size_of::<UsizeLike2>(), size_of::<usize>());
    assert_eq!(size_of::<AtomicUsizeLike>(), size_of::<AtomicUsize>());
}


