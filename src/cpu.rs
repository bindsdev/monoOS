//! Necessary CPU state setup for x86-64.

pub mod gdt;
pub mod idt;

/// Initialize CPU state and structures.
pub fn init() {
    gdt::init();
    idt::init();
}
