//! Enviroment data

use alloc::boxed::Box;

use fs::File;
use slice::Iter;
use string::{String, ToString};
use vec::Vec;

use syscall::sys_chdir;

static mut _args: *mut Vec<&'static str> = 0 as *mut Vec<&'static str>;

/// Arguments
pub fn args<'a>() -> Iter<'a, &'static str> {
    unsafe { (*_args).iter() }
}

/// Initialize arguments
pub unsafe fn args_init(args: Vec<&'static str>) {
    _args = Box::into_raw(box args);
}

/// Destroy arguments
pub unsafe fn args_destroy() {
    if _args as usize > 0 {
        drop(Box::from_raw(_args));
    }
}

/// Method to return the current directory
/// If the current directory cannot be found, None will be returned
pub fn current_dir() -> Result<String, ()> {
    // Return the current path
    if let Some(file) = File::open("") {
        if let Some(path) = file.path() {
            return Ok(path);
        }
    }

    Err(())
}

/// Set the current directory
pub fn set_current_dir(path: &str) -> Result<(), ()> {
    let file_option = if path.is_empty() || path.ends_with('/') {
        File::open(path)
    } else {
        File::open(&(path.to_string() + "/"))
    };

    if let Some(file) = file_option {
        if let Some(file_path) = file.path() {
            let path_c = file_path + "\0";
            if unsafe { sys_chdir(path_c.as_ptr()) } == 0 {
                return Ok(());
            }
        }
    }

    Err(())
}
