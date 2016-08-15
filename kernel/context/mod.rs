//! Context management

/// File operations
pub mod file;

/// Context list
pub static mut CONTEXT: Context = Context::new();

/// A context, which identifies either a process or a thread
#[derive(Copy, Clone, Debug)]
pub struct Context {
    /// The open files in the scheme
    pub files: [Option<file::File>; 32]
}

impl Context {
    pub const fn new() -> Context {
        Context {
            files: [None; 32]
        }
    }

    /// Add a file to the lowest available slot.
    /// Return the file descriptor number or None if no slot was found
    pub fn add_file(&mut self, file: file::File) -> Option<usize> {
        for (i, mut file_option) in self.files.iter_mut().enumerate() {
            if file_option.is_none() {
                *file_option = Some(file);
                return Some(i);
            }
        }
        None
    }
}
