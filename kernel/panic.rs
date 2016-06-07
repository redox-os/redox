use core::fmt;

use syscall;

#[lang="panic_fmt"]
pub extern "C" fn panic_fmt(args: fmt::Arguments, file: &'static str, line: u32) -> ! {
    {
        let contexts = ::env().contexts.lock();
        if let Ok(context) = contexts.current() {
            debugln!("PID {}: {}", context.pid, context.name);

            if let Some(current_syscall) = context.current_syscall {
                debugln!("  SYS {:X}: {} {} {:X} {:X} {:X}", current_syscall.0, current_syscall.1, syscall::name(current_syscall.1), current_syscall.2, current_syscall.3, current_syscall.4);
            }
        }
    }

    debugln!("  KP {}: {}: {}", file, line, args);

    loop {
        unsafe { asm!("cli ; hlt" : : : : "intel", "volatile"); }
    }
}
