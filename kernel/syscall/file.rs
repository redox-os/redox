use arch::context::ContextFile;

use core::slice;

use fs::{ResourceSeek, Url};

use schemes::pipe::{PipeRead, PipeWrite};

use system::c_string_to_str;

use super::{Error, Result, Stat, EBADF, EFAULT, EINVAL, SEEK_CUR, SEEK_END, SEEK_SET};

pub fn do_sys_chdir(path: *const u8) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    unsafe {
        *current.cwd.get() = current.canonicalize(c_string_to_str(path));
    }
    Ok(0)
}

pub fn do_sys_close(fd: usize) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());

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

                return Ok(0);
            }
        }
    }

    Err(Error::new(EBADF))
}

pub fn do_sys_dup(fd: usize) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    let resource = try!(current.get_file(fd));
    let new_resource = try!(resource.dup());
    let new_fd = current.next_fd();
    unsafe {
        (*current.files.get()).push(ContextFile {
            fd: new_fd,
            resource: new_resource,
        });
    }
    Ok(new_fd)
}

pub fn do_sys_fpath(fd: usize, buf: *mut u8, count: usize) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    let resource = try!(current.get_file(fd));
    resource.path(unsafe { slice::from_raw_parts_mut(buf, count) })
}

pub fn do_sys_fstat(fd: usize, stat: *mut Stat) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    let resource = try!(current.get_file(fd));
    if stat as usize > 0 {
        resource.stat(unsafe { &mut *stat })
    } else {
        Err(Error::new(EFAULT))
    }
}

pub fn do_sys_fsync(fd: usize) -> Result<usize> {
    let mut contexts = ::env().contexts.lock();
    let mut current = try!(contexts.current_mut());
    let mut resource = try!(current.get_file_mut(fd));
    resource.sync().and(Ok(0))
}

pub fn do_sys_ftruncate(fd: usize, len: usize) -> Result<usize> {
    let mut contexts = ::env().contexts.lock();
    let mut current = try!(contexts.current_mut());
    let mut resource = try!(current.get_file_mut(fd));
    resource.truncate(len).and(Ok(0))
}

//TODO: Link

pub fn do_sys_lseek(fd: usize, offset: isize, whence: usize) -> Result<usize> {
    let mut contexts = ::env().contexts.lock();
    let mut current = try!(contexts.current_mut());
    let mut resource = try!(current.get_file_mut(fd));
    match whence {
        SEEK_SET => resource.seek(ResourceSeek::Start(offset as usize)),
        SEEK_CUR => resource.seek(ResourceSeek::Current(offset)),
        SEEK_END => resource.seek(ResourceSeek::End(offset)),
        _ => Err(Error::new(EINVAL)),
    }
}

pub fn do_sys_mkdir(path: *const u8, flags: usize) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    let path_string = current.canonicalize(c_string_to_str(path));
    ::env().mkdir(&Url::from_string(path_string), flags).and(Ok(0))
}

pub fn do_sys_open(path: *const u8, flags: usize) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    let path_string = current.canonicalize(c_string_to_str(path));
    let resource = try!(::env().open(&Url::from_string(path_string), flags));
    let fd = current.next_fd();
    unsafe {
        (*current.files.get()).push(ContextFile {
            fd: fd,
            resource: resource,
        });
    }
    Ok(fd)
}

pub fn do_sys_pipe2(fds: *mut usize, _flags: usize) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
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
}

pub fn do_sys_read(fd: usize, buf: *mut u8, count: usize) -> Result<usize> {
    let mut contexts = ::env().contexts.lock();
    let mut current = try!(contexts.current_mut());
    let mut resource = try!(current.get_file_mut(fd));
    resource.read(unsafe { slice::from_raw_parts_mut(buf, count) })
}

pub fn do_sys_unlink(path: *const u8) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    let path_string = current.canonicalize(c_string_to_str(path));
    ::env().unlink(&Url::from_string(path_string)).and(Ok(0))
}

pub fn do_sys_write(fd: usize, buf: *const u8, count: usize) -> Result<usize> {
    let mut contexts = ::env().contexts.lock();
    let mut current = try!(contexts.current_mut());
    let mut resource = try!(current.get_file_mut(fd));
    resource.write(unsafe { slice::from_raw_parts(buf, count) })
}
