use super::{halt, stack_trace};

interrupt!(divide_by_zero, {
    print!("Divide by zero fault\n");
    stack_trace();
    loop { halt(); }
});

interrupt!(debug, {
    print!("Debug trap\n");
});

interrupt!(non_maskable, {
    print!("Non-maskable interrupt\n");
});

interrupt!(breakpoint, {
    print!("Breakpoint trap\n");
});

interrupt!(overflow, {
    print!("Overflow trap\n");
});

interrupt!(bound_range, {
    print!("Bound range exceeded fault\n");
    stack_trace();
    loop { halt(); }
});

interrupt!(invalid_opcode, {
    print!("Invalid opcode fault\n");
    stack_trace();
    loop { halt(); }
});

interrupt!(device_not_available, {
    print!("Device not available fault\n");
    stack_trace();
    loop { halt(); }
});

interrupt_error!(double_fault, {
    print!("Double fault\n");
    stack_trace();
    loop { halt(); }
});

interrupt_error!(invalid_tss, {
    print!("Invalid TSS fault\n");
    stack_trace();
    loop { halt(); }
});

interrupt_error!(segment_not_present, {
    print!("Segment not present fault\n");
    stack_trace();
    loop { halt(); }
});

interrupt_error!(stack_segment, {
    print!("Stack segment fault\n");
    stack_trace();
    loop { halt(); }
});

interrupt_error!(protection, {
    print!("Protection fault\n");
    stack_trace();
    loop { halt(); }
});

interrupt_error!(page, {
    let cr2: usize;
    asm!("mov rax, cr2" : "={rax}"(cr2) : : : "intel", "volatile");
    println!("Page fault: {:>016X}", cr2);
    stack_trace();
    loop { halt(); }
});

interrupt!(fpu, {
    print!("FPU floating point fault\n");
    stack_trace();
    loop { halt(); }
});

interrupt_error!(alignment_check, {
    print!("Alignment check fault\n");
    stack_trace();
    loop { halt(); }
});

interrupt!(machine_check, {
    print!("Machine check fault\n");
    stack_trace();
    loop { halt(); }
});

interrupt!(simd, {
    print!("SIMD floating point fault\n");
    stack_trace();
    loop { halt(); }
});

interrupt!(virtualization, {
    print!("Virtualization fault\n");
    stack_trace();
    loop { halt(); }
});

interrupt_error!(security, {
    print!("Security exception\n");
    stack_trace();
    loop { halt(); }
});
