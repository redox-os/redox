//! # IRQ handling
//!
//! This module defines IRQ handling functions. These functions should all be #[naked],
//! unsafe, extern, and end in `iretq`

/// Interupt Request handler.
#[naked]
pub unsafe extern fn irq() {

}
