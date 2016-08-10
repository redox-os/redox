//! System calls related to process managment.
use arch::context::{context_clone, context_switch, Context, ContextFile};
use arch::regs::Regs;

use collections::{BTreeMap, Vec};
use collections::string::ToString;

use core::{intrinsics, mem};
use core::ops::DerefMut;

use system::{c_array_to_slice, c_string_to_str};
use system::error::{Error, Result, EAGAIN, EACCES, ECHILD, EINVAL};
use system::syscall::{FUTEX_WAKE, FUTEX_WAIT, FUTEX_REQUEUE};

use super::execute::execute;

use fs::SupervisorResource;

pub fn clone(regs: &Regs) -> Result<usize> {
    unsafe { context_clone(regs) }
}

pub fn execve(path: *const u8, args: *const *const u8) -> Result<usize> {
    let mut args_vec = Vec::new();
    args_vec.push(c_string_to_str(path).to_string());
    for arg in c_array_to_slice(args) {
        args_vec.push(c_string_to_str(*arg).to_string());
    }

    execute(args_vec)
}

/// Exit context
pub fn exit(status: usize) -> ! {
    {
        let contexts = unsafe { &mut *::env().contexts.get() };

        let mut statuses = BTreeMap::new();
        let (pid, ppid) = {
            if let Ok(mut current) = contexts.current_mut() {
                mem::swap(&mut statuses, &mut unsafe { current.statuses.inner() }.deref_mut());
                current.exit();
                (current.pid, current.ppid)
            } else {
                (0, 0)
            }
        };

        for mut context in contexts.iter_mut() {
            // Add exit status to parent
            if context.pid == ppid {
                context.statuses.send(pid, status, "exit parent status");
                for (pid, status) in statuses.iter() {
                    context.statuses.send(*pid, *status, "exit child status");
                }
            }

            // Move children to parent
            if context.ppid == pid {
                context.ppid = ppid;
            }
        }
    }

    loop {
        unsafe { context_switch() };
    }
}

pub fn futex(addr: *mut i32, op: usize, val: i32, val2: usize, addr2: *mut i32) -> Result<usize> {
    let contexts = unsafe { &mut *::env().contexts.get() };
    let mut current = contexts.current_mut()?;
    let current_ptr = current.deref_mut() as *mut Context;
    let addr_safe = current.get_ref_mut(addr)?;

    match op {
        FUTEX_WAIT => if unsafe { intrinsics::atomic_load(addr_safe) == val } {
            {
                let futexes = unsafe { &mut *::env().futexes.get() };
                futexes.push_back((addr_safe, current_ptr));
            }

            unsafe { (*current_ptr).block("futex wait") };

            Ok(0)
        } else {
            Err(Error::new(EAGAIN))
        },
        FUTEX_WAKE => {
            let mut woken = 0;

            {
                let futexes = unsafe { &mut *::env().futexes.get() };
                let mut i = 0;
                while i < futexes.len() && (woken as i32) < val {
                    if futexes[i].0 == addr_safe {
                        if let Some(futex) = futexes.swap_remove_back(i) {
                            unsafe { (*futex.1).unblock("futex wake") };
                            woken += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
            }

            Ok(woken)
        },
        FUTEX_REQUEUE => {
            let addr2_safe = current.get_ref_mut(addr2)?;

            let mut woken = 0;
            let mut requeued = 0;

            {
                let futexes = unsafe { &mut *::env().futexes.get() };
                let mut i = 0;
                while i < futexes.len() && (woken as i32) < val {
                    if futexes[i].0 == addr_safe {
                        if let Some(futex) = futexes.swap_remove_back(i) {
                            unsafe { (*futex.1).unblock("futex wake") };
                            woken += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                while i < futexes.len() && requeued < val2 {
                    if futexes[i].0 == addr_safe {
                        futexes[i].0 = addr2_safe;
                        requeued += 1;
                    }
                    i += 1;
                }
            }

            Ok(woken)
        },
        _ => Err(Error::new(EINVAL))
    }
}

pub fn getpid() -> Result<usize> {
    let contexts = unsafe { & *::env().contexts.get() };
    let current = try!(contexts.current());
    Ok(current.pid)
}

#[cfg(target_arch = "x86")]
pub fn iopl(regs: &mut Regs) -> Result<usize> {
    let level = regs.bx;
    if level <= 3 {
        let contexts = unsafe { &mut *::env().contexts.get() };
        let mut current = try!(contexts.current_mut());
        current.iopl = level;

        regs.flags &= 0xFFFFFFFF - 0x3000;
        regs.flags |= (current.iopl << 12) & 0x3000;

        Ok(0)
    } else {
        Err(Error::new(EINVAL))
    }
}

#[cfg(target_arch = "x86_64")]
pub fn iopl(regs: &mut Regs) -> Result<usize> {
    let level = regs.bx;
    if level <= 3 {
        let contexts = unsafe { &mut *::env().contexts.get() };
        let mut current = try!(contexts.current_mut());
        current.iopl = level;

        regs.flags &= 0xFFFFFFFFFFFFFFFF - 0x3000;
        regs.flags |= (current.iopl << 12) & 0x3000;

        Ok(0)
    } else {
        Err(Error::new(EINVAL))
    }
}

//TODO: Finish implementation, add more functions to WaitMap so that matching any or using WNOHANG works
pub fn waitpid(pid: isize, status_ptr: *mut usize, _options: usize) -> Result<usize> {
    let contexts = unsafe { &mut *::env().contexts.get() };
    let current = try!(contexts.current_mut());

    if pid > 0 {
        let status = current.statuses.receive(&(pid as usize), "waitpid status");

        if let Ok(status_safe) = current.get_ref_mut(status_ptr) {
            *status_safe = status;
        }

        Ok(pid as usize)
    } else {
        Err(Error::new(ECHILD))
    }
}

pub fn sched_yield() -> Result<usize> {
    unsafe {
        context_switch();
    }
    Ok(0)
}

/// Supervise a child process of the current context.
///
/// This will make all syscalls the given process makes mark the process as blocked, until it is
/// handled by the supervisor (parrent process) through the returned handle (for details, see the
/// docs in the `system` crate).
///
/// This routine is done by having a field defining whether the process is blocked by a syscall.
/// When the syscall is read from the file handle, this field is set to false, but the process is
/// still stopped (because it is marked as `blocked`), until the new value of the EAX register is
/// written to the file handle.
pub fn supervise(pid: usize) -> Result<usize> {
    let contexts = unsafe { &mut *::env().contexts.get() };
    let cur_pid = try!(contexts.current_mut()).pid;

    let procc;

    {
        let jailed = try!(contexts.find_mut(pid));

        // Make sure that this is actually a child process of the invoker.
        if jailed.ppid != cur_pid {
            return Err(Error::new(EACCES));
        }

        jailed.supervised = true;

        procc = &mut **jailed as *mut _;
    }

    let current = try!(contexts.current_mut());

    let fd = current.next_fd();

    unsafe {
        (*current.files.get()).push(ContextFile {
            fd: fd,
            resource: box try!(SupervisorResource::new(procc)),
        });
    }

    Ok(fd)
}
