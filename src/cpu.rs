//! Necessary CPU state setup for x86-64.

pub mod gdt;
pub mod idt;

pub fn init() {
    gdt::init();
    idt::init();
}
