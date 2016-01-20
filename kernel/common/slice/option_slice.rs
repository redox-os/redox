use collections::range::RangeArgument;
use core::ops::Range;
use core::cmp;

fn bound<T: RangeArgument<Option<usize>>>(len: usize, a: T) -> Range<usize> {
    // These unwraps at the end will NOT panic
    let start = cmp::min(a.start().unwrap_or(&Some(0)).unwrap(), len);
    let end = cmp::min(a.end().unwrap_or(&Some(len)).unwrap(), len);

    if start <= end {
        start..end
    } else {
        0..0
    }
}

pub trait OptionSlice {
    fn option_slice<T: RangeArgument<Option<usize>>>(&self, a: T) -> &Self;
    fn option_slice_mut<T: RangeArgument<Option<usize>>>(&mut self, a: T) -> &mut Self;
}

impl OptionSlice for str {
    fn option_slice<T: RangeArgument<Option<usize>>>(&self, a: T) -> &Self {
        &self[bound(self.len(), a)]
    }

    fn option_slice_mut<T: RangeArgument<Option<usize>>>(&mut self, a: T) -> &mut Self {
        let len = self.len();
        &mut self[bound(len, a)]
    }
}

impl<T> OptionSlice for [T] {
    fn option_slice<U: RangeArgument<Option<usize>>>(&self, a: U) -> &Self {
        &self[bound(self.len(), a)]
    }

    fn option_slice_mut<U: RangeArgument<Option<usize>>>(&mut self, a: U) -> &mut Self {
        let len = self.len();
        &mut self[bound(len, a)]
    }
}
