use syscall::{sys_debug, sys_exit};

pub fn begin_unwind(string: &'static str, file_line: &(&'static str, u32)) -> ! {
    let &(file, line) = file_line;
    println!("{} in {}:{}", string, file, line);

    unsafe {
        sys_exit(-1);
        loop {
            asm!("sti");
            asm!("hlt");
        }
    }
}