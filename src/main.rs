#![no_std]
#![no_main]
#![allow(dead_code, unused)]
#![deny(unsafe_op_in_unsafe_fn, rust_2018_idioms)]
#![feature(
    decl_macro,
    abi_x86_interrupt,
    custom_test_frameworks,
    panic_info_message
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
mod tests;

use core::panic::PanicInfo;
use cpu::idt::hlt;
use limine::FramebufferRequest;

static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new(0);

#[no_mangle]
extern "C" fn kmain() -> ! {
    // Initialize the system logger.
    logger::init();

    // Initialize CPU state.
    cpu::init();
    log::info!("initializing CPU state");

    // Obtain the framebuffer and initialize graphics driver.
    let framebuffer = &*FRAMEBUFFER.get_response().get().unwrap().framebuffers()[0];
    drivers::graphics::init(framebuffer);

    hlt()
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    let location = info.location().unwrap();

    log::error!(
        "panic occurred in {}:{}: {}",
        location.file().strip_prefix("src/").unwrap(),
        location.line(),
        info.message().unwrap()
    );

    hlt()
}
