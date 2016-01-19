use core::slice;
use core::str;
use collections::range::RangeArgument;
use core::ops::Range;
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
    fn get_slice_mut<T: RangeArgument<usize>>(&mut self, a: T) -> &mut Self;
}

fn bound<T: RangeArgument<usize>>(len: usize, a: T) -> Range<usize> {
    let start = min(a.start().map(|&x| x).unwrap_or(0), len - 1);
    let end = min(a.end().map(|&x| x).unwrap_or(len), len);

    if start <= end {
        start..end
    } else {
        0..0
    }
}

impl GetSlice for str {
    fn get_slice<T: RangeArgument<usize>>(&self, a: T) -> &Self {
        &self[bound(self.len(), a)]
    }

    fn get_slice_mut<T: RangeArgument<usize>>(&mut self, a: T) -> &mut Self {
        let len = self.len();
        &mut self[bound(len, a)]
    }
}

impl<T> GetSlice for [T] {
    fn get_slice<U: RangeArgument<usize>>(&self, a: U) -> &Self {
        &self[bound(self.len(), a)]
    }

    fn get_slice_mut<U: RangeArgument<usize>>(&mut self, a: U) -> &mut Self {
        let len = self.len();
        &mut self[bound(len, a)]
    }
}
