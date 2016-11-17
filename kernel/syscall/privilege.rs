use collections::Vec;

use context;
use scheme;
use syscall::error::*;
use syscall::validate::validate_slice;

pub fn getegid() -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let context = context_lock.read();
    Ok(context.egid as usize)
}

pub fn geteuid() -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let context = context_lock.read();
    Ok(context.euid as usize)
}

pub fn getgid() -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let context = context_lock.read();
    Ok(context.rgid as usize)
}

pub fn getuid() -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let context = context_lock.read();
    Ok(context.ruid as usize)
}

pub fn setgid(gid: u32) -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let mut context = context_lock.write();
    if context.egid == 0 {
        context.rgid = gid;
        context.egid = gid;
        Ok(0)
    } else {
        Err(Error::new(EPERM))
    }
}

pub fn setuid(uid: u32) -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let mut context = context_lock.write();
    if context.euid == 0 {
        context.ruid = uid;
        context.euid = uid;
        Ok(0)
    } else {
        Err(Error::new(EPERM))
    }
}

pub fn setns(name_ptrs: &[[usize; 2]]) -> Result<usize> {
    let mut names = Vec::new();
    for name_ptr in name_ptrs {
        names.push(validate_slice(name_ptr[0] as *const u8, name_ptr[1])?);
    }

    let from = {
        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
        let context = context_lock.read();
        context.scheme_ns
    };

    let to = scheme::schemes_mut().setns(from, &names)?;

    {
        let contexts = context::contexts();
        let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
        let mut context = context_lock.write();
        context.scheme_ns = to;
    }

    Ok(0)
}
