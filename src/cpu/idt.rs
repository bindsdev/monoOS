use spin::Lazy;
use x86_64::{
    instructions::interrupts,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

macro add_handlers($idt:expr, $($field:ident)+ ; $($func:ident)+) {
    let idt = &mut $idt;
    $(
        idt.$field.set_handler_fn($func);
    )+
}

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    add_handlers! {
        idt,
        divide_error debug non_maskable_interrupt breakpoint overflow bound_range_exceeded invalid_opcode device_not_available invalid_tss segment_not_present stack_segment_fault general_protection_fault alignment_check ;
        eDE eDB eNMI eBP eOF eBR eUD eNM eTS eNP eSS eGP eAC
    }

    idt
});

macro handler {
    // Normal handler
    ($($ex:ident)+) => ($(
        #[allow(non_snake_case)]
        extern "x86-interrupt" fn $ex(_: ::x86_64::structures::idt::InterruptStackFrame) {
            todo!();
        }
    )+),

    // Handler with error code
    ($($ex:ident)+, ec) => ($(
        #[allow(non_snake_case)]
        extern "x86-interrupt" fn $ex(_: ::x86_64::structures::idt::InterruptStackFrame, _: u64) {
            todo!();
        }
    )+),
}

handler! { eDE eDB eNMI eBP eOF eBR eUD eNM }
handler! { eTS eNP eSS eGP eAC, ec }

#[allow(non_snake_case)]
extern "x86-interrupt" fn eDF(_: InterruptStackFrame, _: u64) -> ! {
    loop {}
}

#[allow(non_snake_case)]
extern "x86-interrupt" fn ePF(_: InterruptStackFrame, _: PageFaultErrorCode) {
    todo!();
}

pub fn hlt() -> ! {
    loop {
        unsafe { x86_64::instructions::hlt() }
    }
}

/// Initialize the IDT and interrupt related facilities.
pub fn init() {
    IDT.load();
    interrupts::enable();
}
