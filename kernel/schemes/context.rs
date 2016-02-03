use alloc::boxed::Box;

use collections::string::String;

use arch::context;

use schemes::{Result, KScheme, Resource, Url, VecResource};

pub struct ContextScheme;

impl KScheme for ContextScheme {
    fn scheme(&self) -> &str {
        "context"
    }

    fn open<'a, 'b: 'a>(&'a mut self, url: Url<'b>, _: usize) -> Result<Box<Resource + 'a>> {
        let mut string = format!("{:<6}{:<6}{:<8}{:<8}{:<6}{:<6}{}",
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

                let line = format!("{:<6}{:<6}{:<8}{:<8}{:<6}{:<6}{}",
                                   context.pid,
                                   context.ppid,
                                   context.slice_total,
                                   memory_string,
                                   unsafe { (*context.files.get()).len() },
                                   flags_string,
                                   context.name);

                string = string + "\n" + &line;
            }
        }

        Ok(box VecResource::new(Url::from_str("context:"), string.into_bytes()))
    }
}
