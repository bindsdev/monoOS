use x86_64::{
    registers::control::Cr2,
    structures::idt::{InterruptStackFrame, PageFaultErrorCode},
};

pub(super) extern "x86-interrupt" fn divide_error(_: InterruptStackFrame) {
    log::error!("division by zero exception");
}

pub(super) extern "x86-interrupt" fn debug(_: InterruptStackFrame) {
    log::error!("debug exception");
}

pub(super) extern "x86-interrupt" fn non_maskable_interrupt(_: InterruptStackFrame) {
    log::error!("non maskable interrupt exception");
}

pub(super) extern "x86-interrupt" fn breakpoint(_: InterruptStackFrame) {
    log::error!("breakpoint exception");
}

pub(super) extern "x86-interrupt" fn overflow(_: InterruptStackFrame) {
    log::error!("overflow exception");
}

pub(super) extern "x86-interrupt" fn bound_range_exceeded(_: InterruptStackFrame) {
    log::error!("bound range exceeded exception");
}

pub(super) extern "x86-interrupt" fn invalid_opcode(_: InterruptStackFrame) {
    log::error!("invalid opcode exception");
}

pub(super) extern "x86-interrupt" fn device_not_available(_: InterruptStackFrame) {
    log::error!("device not available exception");
}

pub(super) extern "x86-interrupt" fn invalid_tss(_: InterruptStackFrame, _: u64) {
    log::error!("invalid TSS exception");
}

pub(super) extern "x86-interrupt" fn segment_not_present(_: InterruptStackFrame, _: u64) {
    log::error!("segment not present exception");
}

pub(super) extern "x86-interrupt" fn stack_segment_fault(_: InterruptStackFrame, _: u64) {
    log::error!("stack segment fault exception");
}

pub(super) extern "x86-interrupt" fn general_protection_fault(_: InterruptStackFrame, _: u64) {
    log::error!("general protection fault exception");
}

pub(super) extern "x86-interrupt" fn alignment_check(_: InterruptStackFrame, _: u64) {
    log::error!("alignment check exception");
}

pub(super) extern "x86-interrupt" fn double_fault(_: InterruptStackFrame, _: u64) -> ! {
    loop {}
}

pub(super) extern "x86-interrupt" fn page_fault(_: InterruptStackFrame, ec: PageFaultErrorCode) {
    log::error!(
        "virtual address {:#X} caused a page fault ({:?})",
        Cr2::read(),
        ec
    );

    super::hlt();
}

pub(super) extern "x86-interrupt" fn timer(_: InterruptStackFrame) {
    unsafe {
        super::PICS
            .lock()
            .notify_end_of_interrupt(super::PIC1_OFFSET);
    }
}
