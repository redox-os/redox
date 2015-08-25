use common::memory::*;

use programs::common::*;

pub struct MemoryScheme;

fn stack_used() -> usize {
    unsafe{
        let stack: u32;
        asm!("" : "={esp}"(stack));
        return (0x200000 - stack) as usize;
    }
}

fn stack_free() -> usize {
    unsafe{
        let stack: u32;
        asm!("" : "={esp}"(stack));
        return (stack - 0x100000) as usize;
    }
}

impl SessionItem for MemoryScheme {
    fn scheme(&self) -> String {
        return "memory".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource>{
        let string = "Memory Used: ".to_string() + memory_used()/1024/1024 + " MB\n"
                   + "Memory Free: " + memory_free()/1024/1024 + " MB\n"
                   + "Stack Used: ".to_string() + stack_used()/1024 + " KB\n"
                   + "Stack Free: ".to_string() + stack_free()/1024 + " KB";
        return box VecResource::new(ResourceType::File, string.to_utf8());
    }
}
