//! X86_64 architecture primitives.

/// Global descriptor table.
pub mod gdt;

/// Interrupt descriptor table.
pub mod idt;

/// IO handling.
pub mod io;

/// IRQ handling.
pub mod irq;

/// Initialization and main function.
pub mod main;

/// Core memory routines.
pub mod mem;

/// Serial driver and `print!` support.
pub mod serial;

/// Task state segment.
pub mod tss;
