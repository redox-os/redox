use interrupt::stack_trace;
use syscall::flag::*;

extern {
    fn ksignal(signal: usize);
}

interrupt_stack!(divide_by_zero, stack, {
    println!("Divide by zero fault at {:>02X}:{:>016X}", stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGFPE);
});

interrupt_stack!(debug, stack, {
    println!("Debug trap at {:>02X}:{:>016X}", stack.cs, stack.rip);
    ksignal(SIGTRAP);
});

interrupt_stack!(non_maskable, stack, {
    println!("Non-maskable interrupt at {:>02X}:{:>016X}", stack.cs, stack.rip);
});

interrupt_stack!(breakpoint, stack, {
    println!("Breakpoint trap at {:>02X}:{:>016X}", stack.cs, stack.rip);
    ksignal(SIGTRAP);
});

interrupt_stack!(overflow, stack, {
    println!("Overflow trap at {:>02X}:{:>016X}", stack.cs, stack.rip);
    ksignal(SIGFPE);
});

interrupt_stack!(bound_range, stack, {
    println!("Bound range exceeded fault at {:>02X}:{:>016X}", stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGSEGV);
});

interrupt_stack!(invalid_opcode, stack, {
    println!("Invalid opcode fault at {:>02X}:{:>016X}", stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGILL);
});

interrupt_stack!(device_not_available, stack, {
    println!("Device not available fault at {:>02X}:{:>016X}", stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGILL);
});

interrupt_error!(double_fault, stack, {
    println!("Double fault: {:X} at {:>02X}:{:>016X}", stack.code, stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGSEGV);
});

interrupt_error!(invalid_tss, stack, {
    println!("Invalid TSS fault: {:X} at {:>02X}:{:>016X}", stack.code, stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGSEGV);
});

interrupt_error!(segment_not_present, stack, {
    println!("Segment not present fault: {:X} at {:>02X}:{:>016X}", stack.code, stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGSEGV);
});

interrupt_error!(stack_segment, stack, {
    println!("Stack segment fault: {:X} at {:>02X}:{:>016X}", stack.code, stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGSEGV);
});

interrupt_error!(protection, stack, {
    println!("Protection fault: {:X} at {:>02X}:{:>016X}", stack.code, stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGSEGV);
});

interrupt_error!(page, stack, {
    let cr2: usize;
    asm!("mov rax, cr2" : "={rax}"(cr2) : : : "intel", "volatile");
    println!("Page fault: {:>02X}:{:>016X} at {:>02X}:{:>016X}", stack.code, cr2, stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGSEGV);
});

interrupt_stack!(fpu, stack, {
    println!("FPU floating point fault at {:>02X}:{:>016X}", stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGFPE);
});

interrupt_error!(alignment_check, stack, {
    println!("Alignment check fault: {:X} at {:>02X}:{:>016X}", stack.code, stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGBUS);
});

interrupt_stack!(machine_check, stack, {
    println!("Machine check fault at {:>02X}:{:>016X}", stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGBUS);
});

interrupt_stack!(simd, stack, {
    println!("SIMD floating point fault at {:>02X}:{:>016X}", stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGFPE);
});

interrupt_stack!(virtualization, stack, {
    println!("Virtualization fault at {:>02X}:{:>016X}", stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGBUS);
});

interrupt_error!(security, stack, {
    println!("Security exception: {:X} at {:>02X}:{:>016X}", stack.code, stack.cs, stack.rip);
    stack_trace();
    ksignal(SIGBUS);
});
