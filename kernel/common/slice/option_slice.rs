use core::slice;
use core::str;
use core::ops::Range;
use core::cmp;

fn option_bound(len: usize, a: Option<usize>, b: Option<usize>) -> Range<usize> {
    let start = cmp::min(a.unwrap_or(0), len);
    let end = cmp::min(a.unwrap_or(len), len);

    if start <= end {
        start..end
    } else {
        0..0
    }
}

pub trait OptionSlice {
    fn option_slice(&self, a: Option<usize>, b: Option<usize>) -> &Self;
    fn option_slice_mut(&mut self, a: Option<usize>, b: Option<usize>) -> &mut Self;
}

impl OptionSlice for str {
    fn option_slice(&self, a: Option<usize>, b: Option<usize>) -> &Self {
        &self[option_bound(self.len(), a, b)]
    }

    fn option_slice_mut(&mut self, a: Option<usize>, b: Option<usize>) -> &mut Self {
        let len = self.len();
        &mut self[option_bound(len, a, b)]
    }
}

impl<T> OptionSlice for [T] {
    fn option_slice(&self, a: Option<usize>, b: Option<usize>) -> &Self {
        &self[option_bound(self.len(), a, b)]
    }

    fn option_slice_mut(&mut self, a: Option<usize>, b: Option<usize>) -> &mut Self {
        let len = self.len();
        &mut self[option_bound(len, a, b)]
    }
}
