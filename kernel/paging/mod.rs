/// A newtype representing a virtual address.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Virtual {
    /// The inner value.
    pub inner: usize,
}
