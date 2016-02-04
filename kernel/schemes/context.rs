use alloc::boxed::Box;

use collections::string::String;

use arch::context;

use fs::{KScheme, Resource, Url, VecResource};

use system::error::Result;

pub struct ContextScheme;

impl KScheme for ContextScheme {
    fn scheme(&self) -> &str {
        "context"
    }

    fn open(&mut self, _: &Url, _: usize) -> Result<Box<Resource>> {
        let mut string = format!("{:<6}{:<6}{:<8}{:<8}{:<6}{:<6}{}\n",
                                 "PID",
                                 "PPID",
                                 "TIME",
                                 "MEM",
                                 "FDS",
                                 "FLG",
                                 "NAME");
        {
            let contexts = ::env().contexts.lock();
            for context in contexts.iter() {
                let mut memory = 0;
                if context.kernel_stack > 0 {
                    memory += context::CONTEXT_STACK_SIZE;
                }
                if let Some(ref stack) = context.stack {
                    memory += stack.virtual_size;
                }
                unsafe {
                    for context_memory in (*context.memory.get()).iter() {
                        memory += context_memory.virtual_size;
                    }
                }

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
                    flags_string.push('U')
                } else {
                    flags_string.push('K');
                }
                if context.interrupted {
                    flags_string.push('I');
                }
                if context.exited {
                    flags_string.push('E');
                }

                string.push_str(&format!("{:<6}{:<6}{:<8}{:<8}{:<6}{:<6}{}\n",
                                   context.pid,
                                   context.ppid,
                                   context.slice_total,
                                   memory_string,
                                   unsafe { (*context.files.get()).len() },
                                   flags_string,
                                   context.name));
            }
        }

        Ok(box VecResource::new("context:", string.into_bytes()))
    }
}
