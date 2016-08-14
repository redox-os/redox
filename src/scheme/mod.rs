/// A scheme is a primitive for handling filesystem syscalls in Redox.
/// Schemes accept paths from the kernel for `open`, and file descriptors that they generate
/// are then passed for operations like `close`, `read`, `write`, etc.
///
/// The kernel validates paths and file descriptors before they are passed to schemes,
/// also stripping the scheme identifier of paths if necessary.
pub trait Scheme {
    /// Open the file at `path` with `flags`.
    /// Returns a file descriptor or an error
    fn open(path: &str, flags: usize) -> Result<usize>;
}
