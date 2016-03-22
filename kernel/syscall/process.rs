use arch::context::{context_clone, context_switch};
use arch::regs::Regs;

use collections::{BTreeMap, Vec};
use collections::string::ToString;

use core::{mem, ptr};
use core::ops::DerefMut;

use system::{c_array_to_slice, c_string_to_str};

use system::error::{Error, Result, ECHILD, EINVAL};

use super::execute::execute;

pub fn do_sys_clone(regs: &Regs) -> Result<usize> {
    unsafe { context_clone(regs) }
}

pub fn do_sys_execve(path: *const u8, args: *const *const u8) -> Result<usize> {
    let mut args_vec = Vec::new();
    args_vec.push(c_string_to_str(path).to_string());
    for arg in c_array_to_slice(args) {
        args_vec.push(c_string_to_str(*arg).to_string());
    }

    execute(args_vec)
}

/// Exit context
///
/// Unsafe due to interrupt disabling and raw pointers
pub fn do_sys_exit(status: usize) -> ! {
    {
        let mut contexts = ::env().contexts.lock();

        let mut statuses = BTreeMap::new();
        let (pid, ppid) = {
            if let Ok(mut current) = contexts.current_mut() {
                current.exited = true;
                mem::swap(&mut statuses, &mut current.statuses.inner.lock().deref_mut());
                (current.pid, current.ppid)
            } else {
                (0, 0)
            }
        };

        for mut context in contexts.iter_mut() {
            // Add exit status to parent
            if context.pid == ppid {
                context.statuses.send(pid, status);
                for (pid, status) in statuses.iter() {
                    context.statuses.send(*pid, *status);
                }
            }

            // Move children to parent
            if context.ppid == pid {
                context.ppid = ppid;
            }
        }
    }

    loop {
        unsafe {
            context_switch();
        }
    }
}

pub fn do_sys_getpid() -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    Ok(current.pid)
}

#[cfg(target_arch = "x86")]
pub fn do_sys_iopl(regs: &mut Regs) -> Result<usize> {
    let level = regs.bx;
    if level <= 3 {
        let mut contexts = ::env().contexts.lock();
        let mut current = try!(contexts.current_mut());
        current.iopl = level;

        regs.flags &= 0xFFFFFFFF - 0x3000;
        regs.flags |= (current.iopl << 12) & 0x3000;

        Ok(0)
    } else {
        Err(Error::new(EINVAL))
    }
}

//TODO: Finish implementation, add more functions to WaitMap so that matching any or using WNOHANG works
pub fn do_sys_waitpid(pid: isize, status_ptr: *mut usize, _options: usize) -> Result<usize> {
    let mut contexts = ::env().contexts.lock();
    let current = try!(contexts.current_mut());

    if pid > 0 {
        let status = current.statuses.receive(&(pid as usize));

        if status_ptr as usize > 0 {
            unsafe {
                ptr::write(status_ptr, status);
            }
        }

        Ok(pid as usize)
    } else {
        Err(Error::new(ECHILD))
    }
}

pub fn do_sys_yield() -> Result<usize> {
    unsafe {
        context_switch();
    }
    Ok(0)
}
