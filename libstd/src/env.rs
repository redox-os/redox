//! Enviroment data

use alloc::boxed::Box;

use fs::File;
use path::{Path, PathBuf};
use io::Result;
use slice::Iter;
use string::{String, ToString};
use vec::Vec;

use system::error::{Error, ENOENT};
use system::syscall::sys_chdir;

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
pub fn current_dir() -> Result<PathBuf> {
    // Return the current path
    match File::open("./") {
        Ok(file) => {
            match file.path() {
                Ok(path) => Ok(path),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

/// Set the current directory
pub fn set_current_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let file_result = if path.as_ref().inner.is_empty() || path.as_ref().inner.ends_with('/') {
        File::open(&path.as_ref().inner)
    } else {
        File::open(&(path.as_ref().inner.to_string() + "/"))
    };

    match file_result {
        Ok(file) => {
            match file.path() {
                Ok(path) => {
                    if let Some(path_str) = path.to_str() {
                        let path_c = path_str.to_string() + "\0";
                        match Error::demux(unsafe { sys_chdir(path_c.as_ptr()) }) {
                            Ok(_) => Ok(()),
                            Err(err) => Err(err),
                        }
                    } else {
                        Err(Error::new(ENOENT))
                    }
                }
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

// TODO: Fully implement `env::var()`
pub fn var(_key: &str) -> Result<String> {
    Ok("This is code filler".to_string())
}
