extern crate core;
extern crate collections;

pub use core::*;
pub use collections;

/*
pub mod prelude {
    pub use ops::{ Drop, Fn, FnMut, FnOnce };
    pub use cmp::{ PartialEq, PartialOrd, Eq, Ord };
    pub use convert::{ AsRef, AsMut, Into, From };
    pub use option::Option::{ self, Some, None };
    pub use option::Result::{ self, Some, None };
    pub use collections::string::{ String, ToString };
    pub use Vec;
    // TODO: Box
    pub use iter::{ Iterator, Extend, IntoIterator, DoubleEndedIterator, ExactSizeIterator };
    pub use mem::drop;
    pub use clone::Clone;
    pub use default::Default;
}
*/

pub mod io;
pub mod fs;
