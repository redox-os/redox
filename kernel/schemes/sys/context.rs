use alloc::boxed::Box;

use collections::string::{String, ToString};

use arch::context;

use fs::{Resource, VecResource};

use system::error::Result;
use system::syscall::MODE_FILE;

pub fn resource() -> Result<Box<Resource>> {
    let mut string = format!("{:<6}{:<6}{:<8}{:<8}{:<8}{:<6}{:<6}{:<6}{}\n",
                             "PID",
                             "PPID",
                             "SWITCH",
                             "TIME",
                             "MEM",
                             "FDS",
                             "FLG",
                             "IOPL",
                             "NAME");
    {
        let contexts = unsafe { & *::env().contexts.get() };
        for context in contexts.iter() {
            let mut memory = 0;
            if context.kernel_stack > 0 {
                memory += context::CONTEXT_STACK_SIZE;
            }
            if let Some(ref stack) = context.stack {
                memory += stack.virtual_size;
            }
            memory += unsafe { (*context.image.get()).size() };
            memory += unsafe { (*context.heap.get()).size() };
            memory += unsafe { (*context.mmap.get()).size() };

            let memory_string = if memory >= 1024 * 1024 * 1024 {
                format!("{} GB", memory / 1024 / 1024 / 1024)
            } else if memory >= 1024 * 1024 {
                format!("{} MB", memory / 1024 / 1024)
            } else if memory >= 1024 {
                format!("{} KB", memory / 1024)
            } else {
                format!("{} B", memory)
            };

            let mut flags_string = String::new();
            if context.stack.is_some() {
                flags_string.push('U');
            } else {
                flags_string.push('K');
            }
            if context.blocked > 0 {
                flags_string.push('B');
            }
            if context.exited {
                flags_string.push('E');
            }
            if context.vfork.is_some() {
                flags_string.push('V');
            }
            if context.wake.is_some() {
                flags_string.push('S');
            }
            if context.supervised {
                flags_string.push('T');
            }

            string.push_str(&format!("{:<6}{:<6}{:<8}{:<8}{:<8}{:<6}{:<6}{:<6}{}\n",
                               context.pid,
                               context.ppid,
                               context.switch,
                               context.time,
                               memory_string,
                               unsafe { (*context.files.get()).len() },
                               flags_string,
                               context.iopl,
                               context.name));
        }
    }

    Ok(box VecResource::new("sys:/context".to_string(), string.into_bytes(), MODE_FILE))
}
