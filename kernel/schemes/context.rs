use alloc::boxed::Box;

use scheduler::context;
use scheduler;

use schemes::{KScheme, Resource, Url, VecResource};

pub struct ContextScheme;

impl KScheme for ContextScheme {
    fn scheme(&self) -> &str {
        "context"
    }

    fn open(&mut self, _: &Url, _: usize) -> Option<Box<Resource>> {
        let mut string = format!("{:<6}{:<6}{:<8}{:<6}{}", "PID", "PPID", "MEM", "FDS", "NAME");
        unsafe {
            let reenable = scheduler::start_no_ints();
            let mut i = 0;
            for context in (*context::contexts_ptr).iter() {
                let mut memory = 0;
                if context.kernel_stack > 0 {
                    memory += context::CONTEXT_STACK_SIZE;
                }
                if let Some(ref stack) = context.stack {
                    memory += stack.virtual_size;
                }
                for context_memory in (*context.memory.get()).iter() {
                    memory += context_memory.virtual_size;
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

                let line = format!("{:<6}{:<6}{:<8}{:<6}{}",
                                   context.pid,
                                   context.ppid,
                                   memory_string,
                                   (*context.files.get()).len(),
                                   context.name);

                string = string + "\n" + &line;
                i += 1;
            }
            scheduler::end_no_ints(reenable);
        }

        Some(box VecResource::new(Url::from_str("context:"), string.into_bytes()))
    }
}
