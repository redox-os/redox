pub trait SliceData<T> {
    fn as_ptr(&self) -> *const T;
    fn len(&self) -> usize;
}

impl SliceData<u8> for str {
    fn as_ptr(&self) -> *const u8 {
        self.as_ptr()
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> SliceData<T> for [T] {
    fn as_ptr(&self) -> *const T {
        self.as_ptr()
    }

    fn len(&self) -> usize {
        self.len()
    }
}
