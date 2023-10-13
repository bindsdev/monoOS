#![no_std]
#![no_main]
#![allow(dead_code, unused)]
#![deny(unsafe_op_in_unsafe_fn, rust_2018_idioms)]
#![feature(
    decl_macro,
    abi_x86_interrupt,
    custom_test_frameworks,
    panic_info_message,
    int_roundings
)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("monoOS is only designed for 64-bit architectures");

#[cfg(not(target_arch = "x86_64"))]
compile_error!("monoOS only supports the x86-64 architecture");

mod cpu;
mod drivers;
mod logger;
mod mem;

#[cfg(test)]
mod tests;

use core::{panic::PanicInfo, sync::atomic::Ordering};
use cpu::idt::hlt;
use limine::{FramebufferRequest, HhdmRequest, KernelAddressRequest, MemmapRequest};

static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new(0);
static HHDM: HhdmRequest = HhdmRequest::new(0);
static MEMMAP: MemmapRequest = MemmapRequest::new(0);

#[no_mangle]
extern "C" fn kmain() -> ! {
    // Initialize the system logger.
    logger::init();

    // Initialize CPU state.
    cpu::init();

    // Initialize memory allocation facilities.
    let physical_memory_offset = HHDM
        .get_response()
        .get()
        .expect("unable to obtain HHDM information")
        .offset;

    mem::PHYSICAL_MEMORY_OFFSET.store(physical_memory_offset, Ordering::Relaxed);

    let memmap = MEMMAP
        .get_response()
        .get_mut()
        .expect("unable to obtain memory map")
        .memmap_mut();

    mem::init(memmap);

    // Obtain the framebuffer an initialize graphics driver.
    // let framebuffer = &*FRAMEBUFFER
    //     .get_response()
    //     .get()
    //     .expect("unable to obtain framebuffer")
    //     .framebuffers()[0];
    // drivers::graphics::init(framebuffer);
    // log::info!("initialized graphics driver");

    #[cfg(test)]
    test_main();

    hlt()
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    log::error!("panic occurred: {}", info.message().unwrap());

    hlt()
}
