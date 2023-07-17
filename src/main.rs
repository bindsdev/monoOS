#![no_std]
#![no_main]
#![allow(dead_code, unused)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("monoOS is only designed for 64-bit architectures");

#[cfg(not(target_arch = "x86_64"))]
compile_error!("monoOS only supports the x86-64 architecture");

mod cpu;
mod graphics;

use core::panic::PanicInfo;
use cpu::idt::hlt;
use limine::FramebufferRequest;

static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new(0);

#[no_mangle]
fn kmain() -> ! {
    // Obtain the framebuffer and setup graphics.
    let framebuffer = &*FRAMEBUFFER.get_response().get().unwrap().framebuffers()[0];
    graphics::init(framebuffer);

    hlt()
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    hlt()
}
