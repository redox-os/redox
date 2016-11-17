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

pub fn setregid(rgid: u32, egid: u32) -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let mut context = context_lock.write();

    if (context.euid == 0
    || rgid as i32 == -1
    || rgid == context.egid
    || rgid == context.rgid)
    && (context.euid == 0
    || egid as i32 == -1
    || egid == context.egid
    || egid == context.rgid)
    {
        if rgid as i32 != -1 {
            context.rgid = rgid;
        }
        if egid as i32 != -1 {
            context.egid = egid;
        }
        Ok(0)
    } else {
        Err(Error::new(EPERM))
    }
}

pub fn setreuid(ruid: u32, euid: u32) -> Result<usize> {
    let contexts = context::contexts();
    let context_lock = contexts.current().ok_or(Error::new(ESRCH))?;
    let mut context = context_lock.write();

    if (context.euid == 0
    || ruid as i32 == -1
    || ruid == context.euid
    || ruid == context.ruid)
    && (context.euid == 0
    || euid as i32 == -1
    || euid == context.euid
    || euid == context.ruid)
    {
        if ruid as i32 != -1 {
            context.ruid = ruid;
        }
        if euid as i32 != -1 {
            context.euid = euid;
        }
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
