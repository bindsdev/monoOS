use core::fmt::{self, Arguments, Write};
use limine::Framebuffer;
use spin::{Mutex, Once};

static FONT: &[u8] = include_bytes!("../tools/ACER710.F08");

#[derive(Debug)]
struct FramebufferInfo {
    byte_len: usize,
    width: usize,
    height: usize,
    pitch: usize,
    bpp: usize,
}

impl FramebufferInfo {
    fn new(byte_len: usize, width: u64, height: u64, pitch: u64, bpp: u16) -> Self {
        Self {
            byte_len,
            width: width as usize,
            height: height as usize,
            pitch: pitch as usize,
            bpp: bpp as usize,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Character {
    c: char,
    fg: u32,
    bg: u32,
}

#[derive(Debug)]
pub struct Writer<'a> {
    fb: &'a mut [u32],
    info: FramebufferInfo,
}

impl<'a> Writer<'a> {
    fn new(fb: &'a mut [u32], info: FramebufferInfo) -> Self {
        Self { fb, info }
    }

    /// Plot a pixel at the given `x` and `y` coordinates with the given `color`
    fn plot_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }

        // We assume that the framebuffer uses an RGB format with 32-bit pixels
        let loc = x + (self.info.pitch / 4) * y;
        self.fb[loc] = color;
    }
}

impl<'a> Write for Writer<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            todo!();
        }

        Ok(())
    }
}

pub static WRITER: Once<Mutex<Writer>> = Once::new();

#[doc(hidden)]
pub fn _print(args: Arguments) {
    WRITER.get().map(|l| l.lock().write_fmt(args));
}

pub fn init(fb: &Framebuffer) {
    let fb_info = FramebufferInfo::new(fb.size(), fb.width, fb.height, fb.pitch, fb.bpp);

    // SAFETY: `as_ptr` will never return `None`
    let fb_addr = unsafe { fb.address.as_ptr().unwrap_unchecked() };
    // SAFETY: `fb_addr` is non-null and aligned
    let fb_slice =
        unsafe { core::slice::from_raw_parts_mut(fb_addr.cast::<u32>(), fb_info.byte_len) };

    let writer = Writer::new(fb_slice, fb_info);

    WRITER.call_once(|| Mutex::new(writer));
}
