//! # The Redox OS Kernel, version 2
//!
//! The Redox OS Kernel is a hybrid kernel that supports X86 systems and
//! provides Unix-like syscalls for primarily Rust applications
//!
//! ## Syscalls
//! Syscalls in Redox are often handled by userspace `schemes`.
//! The essential syscalls in Redox are as follows:
//!
//! ## open(path: &str, flags: usize) -> Result<file_descriptor: usize>
//! Open a file, providing a path as a `&str` and flags, defined elsewhere.
//! Returns a number, known as a file descriptor, that is passed to other syscalls
//!
//! ## close(file_descriptor: usize) -> Result<()>
//! Close a file descriptor, providing the file descriptor from `open`
//! Returns an error, `EBADF`, if the file descriptor was not found.
//! This potential error is often ignored by userspace
//!
//! ## dup(file_descriptor: usize) -> Result<file_descriptor: usize>
//! Duplicate a file descriptor, providing the file descriptor from `open`
//! Returns a new file descriptor, or an error
//!
//! ## read(file_descriptor: usize, buffer: &mut [u8]) -> Result<count: usize>
//! Read from a file descriptor, providing the file descriptor from `open` and a mutable buffer
//! Returns the number of bytes actually read, or an error
//!
//! ## write(file_descriptor: usize, buffer: &[u8]) -> Result<count: usize>
//! Write to a file descriptor, providing the file descriptor from `open` and a const buffer
//! Returns the number of bytes actually written, or an error
//!
//! ## fstat(file_descriptor: usize, stat: &mut Stat) -> Result<()>
//! Get information from a file descriptor, providing the file descriptor from `open`
//! and a mutable Stat struct, defined elsewhere.
//! Returns an error if the operation failed
//!
//! ## fpath(file_descriptor: usize, buffer: &mut [u8]) -> Result<count: usize>
//! Read the path of a file descriptor, providing the file descriptor from `open`
//! and a mutable buffer. The buffer should be 4096 bytes, to ensure that the
//! entire path will fit.
//! Returns the number of bytes actually read, or an error
