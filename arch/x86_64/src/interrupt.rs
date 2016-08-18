//! Interrupt instructions

/// Clear interrupts
#[inline(always)]
pub unsafe fn disable() {
    asm!("cli" : : : : "intel", "volatile");
}

/// Set interrupts
#[inline(always)]
pub unsafe fn enable() {
    asm!("sti" : : : : "intel", "volatile");
}

/// Set interrupts and halt
#[inline(always)]
pub unsafe fn enable_and_halt() {
    asm!("sti
        hlt"
        : : : : "intel", "volatile");
}

/// Halt instruction
#[inline(always)]
pub unsafe fn halt() {
    asm!("hlt" : : : : "intel", "volatile");
}

/// Get a stack trace
//TODO: Check for stack being mapped before dereferencing
#[inline(never)]
pub unsafe fn stack_trace() {
    let mut rbp: usize;
    asm!("xchg bx, bx" : "={rbp}"(rbp) : : : "intel", "volatile");

    println!("TRACE: {:>016X}", rbp);
    //Maximum 64 frames
    for _frame in 0..64 {
        let rip = *(rbp as *const usize).offset(1);
        println!("  {:>016X}: {:>016X}", rbp, rip);
        if rip == 0 {
            break;
        }
        rbp = *(rbp as *const usize);
    }
}

/// x86 External Interrupts (1-16).
pub static EXCEPTIONS: [Descriptor; 21] = [
    Descriptor::new("Division error", Kind::Fault),
    Descriptor::new("Debug trap", Kind::Trap),
    Descriptor::new("Unmaskable interrupt", Kind::Unmaskable),
    Descriptor::new("Breakpoint", Kind::Trap),
    Descriptor::new("Overflow", Kind::Trap),
    Descriptor::new("Out of bound", Kind::Fault),
    Descriptor::new("Invalid opcode", Kind::Fault),
    Descriptor::new("Device unavailable", Kind::Fault),
    Descriptor::new("Double fault", Kind::Fault),
    Descriptor::new("Coprocessor segment overrun", Kind::Fault),
    Descriptor::new("Invalid TSS", Kind::Fault),
    Descriptor::new("Segment not present", Kind::Fault),
    Descriptor::new("Stack-segment fault", Kind::Fault),
    Descriptor::new("General protection", Kind::Fault),
    Descriptor::new("Page fault", Kind::Fault),
    Descriptor::new("Reserved", Kind::Reserved),
    Descriptor::new("x87 FPU", Kind::Fault),
    Descriptor::new("Unaligned memory access", Kind::Fault),
    Descriptor::new("Machine check", Kind::Abort),
    Descriptor::new("SIMD floating-point", Kind::Fault),
    Descriptor::new("Virtualization violation", Kind::Fault),
];

/// An interrupt description.
#[derive(Debug, Copy, Clone)]
pub struct Descriptor {
    /// The description of this interrupt.
    pub desc: &'static str,
    /// The interrupt type.
    pub kind: Kind,
}

impl Descriptor {
    /// Create a new interrupt description.
    pub const fn new(desc: &'static str, kind: Kind) -> Descriptor {
        Descriptor {
            desc: desc,
            kind: kind,
        }
    }
}

/// The interrupt kind.
#[derive(Debug, Copy, Clone)]
pub enum Kind {
    /// A fault.
    ///
    /// This can have multiple sources, but is often a result of a program error of some sort.
    Fault,
    /// A trap.
    ///
    /// These are often for debugging purposes.
    Trap,
    /// A deliberate abort.
    Abort,
    /// An unmaskable interrupt.
    ///
    /// This is a forced interrupt which need to be handled immediately.
    Unmaskable,
    /// Reserved or deprecated.
    Reserved,
}
