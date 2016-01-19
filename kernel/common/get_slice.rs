use core::slice;
use core::str;
use collections::range::RangeArgument;
use core::cmp::{max, min};

/// Bounded slice abstraction
///
/// # Code Migration
///
/// `foo[a..b]` => `foo.get_slice(a..b)`
///
/// `foo[a..]` => `foo.get_slice(a..)`
///
/// `foo[..b]` => `foo.get_slice(..b)`
///
pub trait GetSlice {
    fn get_slice<T: RangeArgument<usize>>(&self, a: T) -> &Self;
}

impl GetSlice for str {
    fn get_slice<T: RangeArgument<usize>>(&self, a: T) -> &Self {
        let start = min(a.start().map(|&x| x).unwrap_or(self.len() - 1), self.len() - 1);
        let end = min(a.end().map(|&x| x).unwrap_or(self.len() - 1), self.len() - 1);

        if start <= end {
            &self[start..end + 1]
        } else {
            ""
        }
    }
}

impl<T> GetSlice for [T] {
    fn get_slice<U: RangeArgument<usize>>(&self, a: U) -> &Self {
        let start = min(a.start().map(|&x| x).unwrap_or(self.len() - 1), self.len() - 1);
        let end = min(a.end().map(|&x| x).unwrap_or(self.len() - 1), self.len() - 1);

        if start <= end {
            &self[start..end + 1]
        } else {
            &[]
        }
    }
}
