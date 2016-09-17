use core::slice;

use super::Result;

/// Convert a pointer and length to slice, if valid
/// TODO: Check validity
pub fn validate_slice<T>(ptr: *const T, len: usize) -> Result<&'static [T]> {
    Ok(unsafe { slice::from_raw_parts(ptr, len) })
}

/// Convert a pointer and length to slice, if valid
/// TODO: Check validity
pub fn validate_slice_mut<T>(ptr: *mut T, len: usize) -> Result<&'static mut [T]> {
    Ok(unsafe { slice::from_raw_parts_mut(ptr, len) })
}
