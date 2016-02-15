use arch::context::ContextFile;

use core::slice;

use fs::{ResourceSeek, Url};

use schemes::pipe::{PipeRead, PipeWrite};

use system::c_string_to_str;

use super::{Error, EBADF, EFAULT, EINVAL, ESRCH, SEEK_CUR, SEEK_END, SEEK_SET};

pub fn do_sys_chdir(path: *const u8) -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        unsafe {
            *current.cwd.get() = current.canonicalize(c_string_to_str(path));
        }
        Ok(0)
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_close(fd: usize) -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        let mut ret = Err(Error::new(EBADF));

        for i in 0..unsafe { (*current.files.get()).len() } {
            let mut remove = false;
            if let Some(file) = unsafe { (*current.files.get()).get(i) } {
                if file.fd == fd {
                    remove = true;
                }
            }

            if remove {
                if i < unsafe { (*current.files.get()).len() } {
                    drop(unsafe { (*current.files.get()).remove(i) });

                    ret = Ok(0);
                }

                break;
            }
        }

        ret
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_dup(fd: usize) -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        if let Some(resource) = unsafe { current.get_file(fd) } {
            match resource.dup() {
                Ok(new_resource) => {
                    let new_fd = unsafe { current.next_fd() };

                    unsafe {
                        (*current.files.get()).push(ContextFile {
                            fd: new_fd,
                            resource: new_resource,
                        });
                    }

                    Ok(new_fd)
                }
                Err(err) => Err(err),
            }
        } else {
            Err(Error::new(EBADF))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_fpath(fd: usize, buf: *mut u8, count: usize) -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        if let Some(resource) = unsafe { current.get_file(fd) } {
            resource.path(unsafe { slice::from_raw_parts_mut(buf, count) })
        } else {
            Err(Error::new(EBADF))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_fsync(fd: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    Error::mux(if let Some(mut current) = contexts.current_mut() {
        if let Some(mut resource) = unsafe { current.get_file_mut(fd) } {
            match resource.sync() {
                Ok(_) => Ok(0),
                Err(err) => Err(err),
            }
        } else {
            Err(Error::new(EBADF))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_ftruncate(fd: usize, len: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    Error::mux(if let Some(mut current) = contexts.current_mut() {
        if let Some(mut resource) = unsafe { current.get_file_mut(fd) } {
            match resource.truncate(len) {
                Ok(_) => Ok(0),
                Err(err) => Err(err),
            }
        } else {
            Err(Error::new(EBADF))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

//TODO: Link

pub fn do_sys_lseek(fd: usize, offset: isize, whence: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    Error::mux(if let Some(mut current) = contexts.current_mut() {
        if let Some(mut resource) = unsafe { current.get_file_mut(fd) } {
            match whence {
                SEEK_SET => resource.seek(ResourceSeek::Start(offset as usize)),
                SEEK_CUR => resource.seek(ResourceSeek::Current(offset)),
                SEEK_END => resource.seek(ResourceSeek::End(offset)),
                _ => Err(Error::new(EINVAL)),
            }
        } else {
            Err(Error::new(EBADF))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_mkdir(path: *const u8, flags: usize) -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        let path_string = current.canonicalize(c_string_to_str(path));

        match (::env()).mkdir(&Url::from_string(path_string), flags) {
            Ok(_) => Ok(0),
            Err(err) => Err(err),
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_open(path: *const u8, flags: usize) -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        let path_string = current.canonicalize(c_string_to_str(path));

        match (::env()).open(&Url::from_string(path_string), flags) {
            Ok(resource) => {
                let fd = unsafe { current.next_fd() };

                unsafe {
                    (*current.files.get()).push(ContextFile {
                        fd: fd,
                        resource: resource,
                    });
                }

                Ok(fd)
            }
            Err(err) => Err(err),
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_pipe2(fds: *mut usize, _flags: usize) -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        if fds as usize > 0 {
            let read = box PipeRead::new();
            let write = box PipeWrite::new(&read);

            unsafe {
                *fds.offset(0) = current.next_fd();
                (*current.files.get()).push(ContextFile {
                    fd: *fds.offset(0),
                    resource: read,
                });

                *fds.offset(1) = current.next_fd();
                (*current.files.get()).push(ContextFile {
                    fd: *fds.offset(1),
                    resource: write,
                });
            }

            Ok(0)
        } else {
            Err(Error::new(EFAULT))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_read(fd: usize, buf: *mut u8, count: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    Error::mux(if let Some(mut current) = contexts.current_mut() {
        if let Some(resource) = unsafe { current.get_file_mut(fd) } {
            resource.read(unsafe { slice::from_raw_parts_mut(buf, count) })
        } else {
            Err(Error::new(EBADF))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_unlink(path: *const u8) -> usize {
    let contexts = ::env().contexts.lock();
    Error::mux(if let Some(current) = contexts.current() {
        let path_string = current.canonicalize(c_string_to_str(path));

        match (::env()).unlink(&Url::from_string(path_string)) {
            Ok(_) => Ok(0),
            Err(err) => Err(err),
        }
    } else {
        Err(Error::new(ESRCH))
    })
}

pub fn do_sys_write(fd: usize, buf: *const u8, count: usize) -> usize {
    let mut contexts = ::env().contexts.lock();
    Error::mux(if let Some(mut current) = contexts.current_mut() {
        if let Some(resource) = unsafe { current.get_file_mut(fd) } {
            resource.write(unsafe { slice::from_raw_parts(buf, count) })
        } else {
            Err(Error::new(EBADF))
        }
    } else {
        Err(Error::new(ESRCH))
    })
}
