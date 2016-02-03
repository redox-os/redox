use core::slice;
use core::str;

/// Bounded slice abstraction
///
/// # Code Migration
///
/// `foo[a..b]` => `foo.get_slice(Some(a), Some(b))`
///
/// `foo[a..]` => `foo.get_slice(Some(a), None)`
///
/// `foo[..b]` => `foo.get_slice(None, Some(b))`
///
pub trait GetSlice {
    fn get_slice(&self, a: Option<usize>, b: Option<usize>) -> &Self; }

impl GetSlice for str {
    fn get_slice(&self, a: Option<usize>, b: Option<usize>) -> &Self {
        let slice = unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) };
        let a = if let Some(tmp) = a {
            let len = slice.len();
            if tmp > len {
                len
            } else {
                tmp
            }
        } else {
            0
        };
        let b = if let Some(tmp) = b {
            let len = slice.len();
            if tmp > len {
                len
            } else {
                tmp
            }
        } else {
            slice.len()
        };

        if a >= b {
            return "";
        }

        unsafe { str::from_utf8_unchecked(&slice[a..b]) }
    }
}

impl<T> GetSlice for [T] {
    fn get_slice(&self, a: Option<usize>, b: Option<usize>) -> &Self {
        let slice = unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) };
        let a = if let Some(tmp) = a {
            let len = slice.len();
            if tmp > len {
                len
            } else {
                tmp
            }
        } else {
            0
        };
        let b = if let Some(tmp) = b {
            let len = slice.len();
            if tmp > len {
                len
            } else {
                tmp
            }
        } else {
            slice.len()
        };

        if a >= b {
            return &[];
        }

        &slice[a..b]
    }
}
