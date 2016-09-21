use core::{mem, slice};

use arch::paging::{ActivePageTable, Page, VirtualAddress, entry};
use syscall::error::*;

fn validate(address: usize, size: usize, flags: entry::EntryFlags) -> Result<()> {
    let active_table = unsafe { ActivePageTable::new() };

    let start_page = Page::containing_address(VirtualAddress::new(address));
    let end_page = Page::containing_address(VirtualAddress::new(address + size - 1));
    for page in Page::range_inclusive(start_page, end_page) {
        let page_flags = active_table.translate_page_flags(page).ok_or(Error::new(EFAULT))?;
        if ! page_flags.contains(flags) {
            return Err(Error::new(EFAULT));
        }
    }

    Ok(())
}

/// Convert a pointer and length to slice, if valid
pub fn validate_slice<T>(ptr: *const T, len: usize) -> Result<&'static [T]> {
    if len == 0 {
        Ok(&[])
    } else {
        validate(ptr as usize, len * mem::size_of::<T>(), entry::PRESENT /* TODO | entry::USER_ACCESSIBLE */)?;
        Ok(unsafe { slice::from_raw_parts(ptr, len) })
    }
}

/// Convert a pointer and length to slice, if valid
pub fn validate_slice_mut<T>(ptr: *mut T, len: usize) -> Result<&'static mut [T]> {
    if len == 0 {
        Ok(&mut [])
    } else {
        validate(ptr as usize, len * mem::size_of::<T>(), entry::PRESENT | entry::WRITABLE /* TODO | entry::USER_ACCESSIBLE */)?;
        Ok(unsafe { slice::from_raw_parts_mut(ptr, len) })
    }
}
