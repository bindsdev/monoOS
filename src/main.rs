#![no_std]
#![no_main]
#![allow(dead_code)]

mod cpu;
mod graphics;

use core::panic::PanicInfo;
use limine::FramebufferRequest;

#[cfg(not(target_pointer_width = "64"))]
compile_error!("monoOS is only designed for 64-bit architectures");

#[cfg(not(target_arch = "x86_64"))]
compile_error!("monoOS only supports the x86_64 architecture");

static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new(0);

fn kmain() -> ! {
    // Obtain the framebuffer and setup graphics.
    let framebuffer = &*FRAMEBUFFER.get_response().get().unwrap().framebuffers()[0];
    graphics::init(framebuffer);

    loop {}
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
