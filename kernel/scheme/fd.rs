/// A file descriptor.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fd {
    inner: usize,
}
