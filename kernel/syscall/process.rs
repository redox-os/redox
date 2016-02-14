use alloc::arc::Arc;

use arch::context::{context_clone, context_switch, Context, ContextStatus};

use collections::Vec;
use collections::string::ToString;

use core::{mem, ptr, usize};
use core::ops::Deref;

use system::{c_array_to_slice, c_string_to_str};

use super::{Error, Result, CLONE_VFORK, ECHILD, ESRCH};
use super::execute::execute;

pub fn do_sys_clone(flags: usize) -> Result<usize> {
    let mut clone_pid = usize::MAX;
    let mut mem_count = 0;

    {
        let mut contexts = ::env().contexts.lock();

        let child_option = if let Ok(parent) = contexts.current() {
            clone_pid = unsafe { Context::next_pid() };
            mem_count = Arc::strong_count(&parent.memory);

            let parent_ptr: *const Context = parent.deref();

            let mut context_clone_args: Vec<usize> = Vec::new();
            context_clone_args.push(clone_pid);
            context_clone_args.push(flags);
            context_clone_args.push(parent_ptr as usize);
            context_clone_args.push(0); //Return address, 0 catches bad code

            Some(unsafe {
                Context::new(format!("kclone {}", parent.name),
                             context_clone as usize,
                             &context_clone_args)
            })
        } else {
            None
        };

        if let Some(child) = child_option {
            unsafe { contexts.push(child) };
        }
    }

    unsafe {
        context_switch(false);
    }

    if clone_pid != usize::MAX {
        let contexts = ::env().contexts.lock();
        let current = try!(contexts.current());
        if current.pid == clone_pid {
            Ok(0)
        } else {
            if flags & CLONE_VFORK == CLONE_VFORK {
                while Arc::strong_count(&current.memory) > mem_count {
                    unsafe {
                        context_switch(false);
                    }
                }
            }
            Ok(clone_pid)
        }
    } else {
        Err(Error::new(ESRCH))
    }
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
    //TODO: DOUBLE CHECK THIS FUNCTION
    {
        let mut contexts = ::env().contexts.lock();

        let mut statuses = Vec::new();
        let (pid, ppid) = {
            if let Ok(mut current) = contexts.current_mut() {
                current.exited = true;
                mem::swap(&mut statuses, &mut current.statuses);
                (current.pid, current.ppid)
            } else {
                (0, 0)
            }
        };

        for mut context in contexts.iter_mut() {
            // Add exit status to parent
            if context.pid == ppid {
                context.statuses.push(ContextStatus {
                    pid: pid,
                    status: status,
                });
                for status in statuses.iter() {
                    context.statuses.push(ContextStatus {
                        pid: status.pid,
                        status: status.status,
                    });
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
            context_switch(false);
        }
    }
}

pub fn do_sys_getpid() -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    Ok(current.pid)
}

pub fn do_sys_waitpid(pid: isize, status: *mut usize, _options: usize) -> Result<usize> {
    let mut ret = Err(Error::new(ECHILD));

    loop {
        {
            let mut contexts = ::env().contexts.lock();
            if let Ok(mut current) = contexts.current_mut() {
                let mut found = false;
                let mut i = 0;
                while i < current.statuses.len() {
                    if let Some(current_status) = current.statuses.get(i) {
                        if pid > 0 && pid as usize == current_status.pid {
                            // Specific child
                            found = true;
                        } else if pid == 0 {
                            // TODO Any child whose PGID is equal to this process
                        } else if pid == -1 {
                            // Any child
                            found = true;
                        } else {
                            // TODO Any child whose PGID is equal to abs(pid)
                        }
                    }
                    if found {
                        let current_status = current.statuses.remove(i);

                        ret = Ok(current_status.pid);
                        if status as usize > 0 {
                            unsafe {
                                ptr::write(status, current_status.status);
                            }
                        }

                        break;
                    } else {
                        i += 1;
                    }
                }
                if found {
                    break;
                }
            } else {
                break;
            }
        }

        unsafe {
            context_switch(false);
        }
    }

    ret
}

pub fn do_sys_yield() -> Result<usize> {
    unsafe {
        context_switch(false);
    }
    Ok(0)
}
