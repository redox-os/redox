pub mod get_slice;

pub use self::get_slice::GetSlice;

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
