//! Interrupt instructions

/// Clear interrupts
#[inline(always)]
pub unsafe fn disable_interrupts() {
    asm!("cli" : : : : "intel", "volatile");
}

/// Set interrupts
#[inline(always)]
pub unsafe fn enable_interrupts() {
    asm!("sti" : : : : "intel", "volatile");
}

/// Halt instruction
#[inline(always)]
pub unsafe fn halt() {
    asm!("hlt" : : : : "intel", "volatile");
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
