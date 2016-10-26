//! Filesystem syscalls
use core::sync::atomic::Ordering;

use context;
use scheme;
use syscall::data::Packet;
use syscall::error::*;

pub fn file_op(a: usize, fd: usize, c: usize, d: usize) -> Result<usize> {
    let (file, pid, uid, gid) = {
        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
        let context = context_lock.read();
        let file = context.get_file(fd).ok_or(Error::new(EBADF))?;
        (file, context.id, context.euid, context.egid)
    };

    let scheme = {
        let schemes = scheme::schemes();
        let scheme = schemes.get(file.scheme).ok_or(Error::new(EBADF))?;
        scheme.clone()
    };

    let mut packet = Packet {
        id: 0,
        pid: pid,
        uid: uid,
        gid: gid,
        a: a,
        b: file.number,
        c: c,
        d: d
    };

    scheme.handle(&mut packet);

    Error::demux(packet.a)
}

pub fn file_op_slice(a: usize, fd: usize, slice: &[u8]) -> Result<usize> {
    file_op(a, fd, slice.as_ptr() as usize, slice.len())
}

pub fn file_op_mut_slice(a: usize, fd: usize, slice: &mut [u8]) -> Result<usize> {
    file_op(a, fd, slice.as_mut_ptr() as usize, slice.len())
}

/// Change the current working directory
pub fn chdir(path: &[u8]) -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let context = context_lock.read();
    let canonical = context.canonicalize(path);
    *context.cwd.lock() = canonical;
    Ok(0)
}

/// Get the current working directory
pub fn getcwd(buf: &mut [u8]) -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let context = context_lock.read();
    let cwd = context.cwd.lock();
    let mut i = 0;
    while i < buf.len() && i < cwd.len() {
        buf[i] = cwd[i];
        i += 1;
    }
    Ok(i)
}

/// Open syscall
pub fn open(path: &[u8], flags: usize) -> Result<usize> {
    let (path_canon, uid, gid) = {
        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
        let context = context_lock.read();
        (context.canonicalize(path), context.euid, context.egid)
    };

    let mut parts = path_canon.splitn(2, |&b| b == b':');
    let namespace_opt = parts.next();
    let reference_opt = parts.next();

    let (scheme_id, file_id) = {
        let namespace = namespace_opt.ok_or(Error::new(ENODEV))?;
        let (scheme_id, scheme) = {
            let schemes = scheme::schemes();
            let (scheme_id, scheme) = schemes.get_name(namespace).ok_or(Error::new(ENODEV))?;
            (scheme_id, scheme.clone())
        };
        let file_id = scheme.open(reference_opt.unwrap_or(b""), flags, uid, gid)?;
        (scheme_id, file_id)
    };

    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let context = context_lock.read();
    context.add_file(::context::file::File {
        scheme: scheme_id,
        number: file_id,
        event: None,
    }).ok_or(Error::new(EMFILE))
}

pub fn pipe2(fds: &mut [usize], flags: usize) -> Result<usize> {
    if fds.len() >= 2 {
        let scheme_id = ::scheme::pipe::PIPE_SCHEME_ID.load(Ordering::SeqCst);
        let (read_id, write_id) = ::scheme::pipe::pipe(flags);

        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
        let context = context_lock.read();

        let read_fd = context.add_file(::context::file::File {
            scheme: scheme_id,
            number: read_id,
            event: None,
        }).ok_or(Error::new(EMFILE))?;

        let write_fd = context.add_file(::context::file::File {
            scheme: scheme_id,
            number: write_id,
            event: None,
        }).ok_or(Error::new(EMFILE))?;

        fds[0] = read_fd;
        fds[1] = write_fd;

        Ok(0)
    } else {
        Err(Error::new(EFAULT))
    }
}

/// mkdir syscall
pub fn mkdir(path: &[u8], mode: u16) -> Result<usize> {
    let (path_canon, uid, gid) = {
        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
        let context = context_lock.read();
        (context.canonicalize(path), context.euid, context.egid)
    };

    let mut parts = path_canon.splitn(2, |&b| b == b':');
    let namespace_opt = parts.next();
    let reference_opt = parts.next();

    let namespace = namespace_opt.ok_or(Error::new(ENODEV))?;
    let scheme = {
        let schemes = scheme::schemes();
        let (_scheme_id, scheme) = schemes.get_name(namespace).ok_or(Error::new(ENODEV))?;
        scheme.clone()
    };
    scheme.mkdir(reference_opt.unwrap_or(b""), mode, uid, gid)
}

/// rmdir syscall
pub fn rmdir(path: &[u8]) -> Result<usize> {
    let (path_canon, uid, gid) = {
        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
        let context = context_lock.read();
        (context.canonicalize(path), context.euid, context.egid)
    };

    let mut parts = path_canon.splitn(2, |&b| b == b':');
    let namespace_opt = parts.next();
    let reference_opt = parts.next();

    let namespace = namespace_opt.ok_or(Error::new(ENODEV))?;
    let scheme = {
        let schemes = scheme::schemes();
        let (_scheme_id, scheme) = schemes.get_name(namespace).ok_or(Error::new(ENODEV))?;
        scheme.clone()
    };
    scheme.rmdir(reference_opt.unwrap_or(b""), uid, gid)
}

/// Unlink syscall
pub fn unlink(path: &[u8]) -> Result<usize> {
    let (path_canon, uid, gid) = {
        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
        let context = context_lock.read();
        (context.canonicalize(path), context.euid, context.egid)
    };

    let mut parts = path_canon.splitn(2, |&b| b == b':');
    let namespace_opt = parts.next();
    let reference_opt = parts.next();

    let namespace = namespace_opt.ok_or(Error::new(ENODEV))?;
    let scheme = {
        let schemes = scheme::schemes();
        let (_scheme_id, scheme) = schemes.get_name(namespace).ok_or(Error::new(ENODEV))?;
        scheme.clone()
    };
    scheme.unlink(reference_opt.unwrap_or(b""), uid, gid)
}

/// Close syscall
pub fn close(fd: usize) -> Result<usize> {
    let file = {
        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
        let context = context_lock.read();
        let file = context.remove_file(fd).ok_or(Error::new(EBADF))?;
        file
    };

    if let Some(event_id) = file.event {
        context::event::unregister(fd, file.scheme, event_id);
    }

    let scheme = {
        let schemes = scheme::schemes();
        let scheme = schemes.get(file.scheme).ok_or(Error::new(EBADF))?;
        scheme.clone()
    };
    scheme.close(file.number)
}

/// Duplicate file descriptor
pub fn dup(fd: usize, buf: &[u8]) -> Result<usize> {
    let file = {
        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
        let context = context_lock.read();
        let file = context.get_file(fd).ok_or(Error::new(EBADF))?;
        file
    };

    let new_id = {
        let scheme = {
            let schemes = scheme::schemes();
            let scheme = schemes.get(file.scheme).ok_or(Error::new(EBADF))?;
            scheme.clone()
        };
        scheme.dup(file.number, buf)?
    };

    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let context = context_lock.read();
    context.add_file(::context::file::File {
        scheme: file.scheme,
        number: new_id,
        event: None,
    }).ok_or(Error::new(EMFILE))
}

/// Register events for file
pub fn fevent(fd: usize, flags: usize) -> Result<usize> {
    let file = {
        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
        let context = context_lock.read();
        let mut files = context.files.lock();
        let mut file = files.get_mut(fd).ok_or(Error::new(EBADF))?.ok_or(Error::new(EBADF))?;
        if let Some(event_id) = file.event.take() {
            println!("{}: {}:{}: events already registered: {}", fd, file.scheme, file.number, event_id);
            context::event::unregister(fd, file.scheme, event_id);
        }
        file.clone()
    };

    let scheme = {
        let schemes = scheme::schemes();
        let scheme = schemes.get(file.scheme).ok_or(Error::new(EBADF))?;
        scheme.clone()
    };
    let event_id = scheme.fevent(file.number, flags)?;
    {
        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
        let context = context_lock.read();
        let mut files = context.files.lock();
        let mut file = files.get_mut(fd).ok_or(Error::new(EBADF))?.ok_or(Error::new(EBADF))?;
        file.event = Some(event_id);
    }
    context::event::register(fd, file.scheme, event_id);
    Ok(0)
}
