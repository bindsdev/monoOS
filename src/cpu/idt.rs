mod handlers;

use complete_pic::pic8259::ChainedPics;
use handlers::*;
use spin::{Lazy, Mutex};
use x86_64::{instructions::interrupts, structures::idt::InterruptDescriptorTable};

macro exception_handlers($idt:expr, $($exception:ident)+) {
    let idt = &mut $idt;
    $(
        idt.$exception.set_handler_fn($exception);
    )+
}

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    exception_handlers! {
        idt,
        divide_error debug non_maskable_interrupt breakpoint overflow bound_range_exceeded invalid_opcode device_not_available invalid_tss segment_not_present stack_segment_fault general_protection_fault alignment_check double_fault page_fault
    }

    idt[PIC1_OFFSET as usize].set_handler_fn(timer);

    idt
});

const PIC1_OFFSET: u8 = 32;
const PIC2_OFFSET: u8 = PIC1_OFFSET + 8;
static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET) });

pub fn hlt() -> ! {
    loop {
        x86_64::instructions::hlt()
    }
}

/// Initialize the IDT and interrupt related facilities.
pub fn init() {
    IDT.load();
    log::info!("initialized IDT");

    interrupts::without_interrupts(|| {
        let mut pics = PICS.lock();

        unsafe {
            pics.initialize();
            pics.unmask();
        }
    });

    log::info!(
        "initialized 8259 PIC with master offset {m:#X} and slave offset {s:#X}",
        m = PIC1_OFFSET,
        s = PIC2_OFFSET
    );

    interrupts::enable();
}
