pub trait AsOption<T> {
    fn as_option(&self) -> Option<T>;
}

impl AsOption<usize> for usize {
    fn as_option(&self) -> Option<usize> {
        Some(*self)
    }
}

impl AsOption<usize> for Option<usize> {
    fn as_option(&self) -> Option<usize> {
        *self
    }
}

use core_collections::range::RangeArgument;
use core::ops::Range;
use core::cmp;

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
    fn get_slice<T: AsOption<usize>, U: RangeArgument<T>>(&self, a: U) -> &Self;
    fn get_slice_mut<T: AsOption<usize>, U: RangeArgument<T>>(&mut self, a: U) -> &mut Self;
}

fn bound<T: AsOption<usize>, U: RangeArgument<T>>(len: usize, a: U) -> Range<usize> {
    let start = cmp::min(a.start().map(|x| x.as_option()).unwrap_or(Some(0)).unwrap(), len);
    let end = cmp::min(a.end().map(|x| x.as_option()).unwrap_or(Some(len)).unwrap(), len);

    if start <= end {
        start..end
    } else {
        0..0
    }
}

impl GetSlice for str {
    fn get_slice<T: AsOption<usize>, U: RangeArgument<T>>(&self, a: U) -> &Self {
        &self[bound(self.len(), a)]
    }

    fn get_slice_mut<T: AsOption<usize>, U: RangeArgument<T>>(&mut self, a: U) -> &mut Self {
        let len = self.len();
        &mut self[bound(len, a)]
    }
}

impl<T> GetSlice for [T] {
    fn get_slice<U: AsOption<usize>, V: RangeArgument<U>>(&self, a: V) -> &Self {
        &self[bound(self.len(), a)]
    }

    fn get_slice_mut<U: AsOption<usize>, V: RangeArgument<U>>(&mut self, a: V) -> &mut Self {
        let len = self.len();
        &mut self[bound(len, a)]
    }
}
