//! Graphics driver using the Limine framebuffer

use core::fmt::{self, Arguments, Write};
use limine::Framebuffer;
use spin::{Mutex, Once};

const FONT_WIDTH: usize = 8;
const FONT_HEIGHT: usize = 16;
const DEFAULT_FG: u32 = u32::MAX;
const DEFAULT_BG: u32 = 0xAAAAAA;
static FONT: &[[u8; FONT_HEIGHT]; 256] =
    unsafe { &core::mem::transmute(*include_bytes!("../tools/COMPAQTH.F16")) };

#[derive(Debug)]
struct FramebufferInfo {
    /// Width of the framebuffer in pixels
    width: usize,

    /// Height of the framebuffer in pixels
    height: usize,

    /// The number of bytes you have to skip to go down a pixel
    pitch: usize,

    /// The amount of bits per pixel
    bpp: usize,
}

impl FramebufferInfo {
    fn new(width: u64, height: u64, pitch: u64, bpp: u16) -> Self {
        Self {
            width: width as usize,
            height: height as usize,
            pitch: pitch as usize,
            bpp: bpp as usize,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct ColorCode(u32, u32);

impl ColorCode {
    /// Get the foreground color of the color code
    fn fg(&self) -> u32 {
        self.0
    }

    /// Get the background color of the color code
    fn bg(&self) -> u32 {
        self.1
    }
}

impl Default for ColorCode {
    fn default() -> Self {
        Self(DEFAULT_FG, DEFAULT_BG)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
struct Character {
    ch: char,
    color: ColorCode,
}

#[derive(Debug)]
pub struct Writer {
    /// Slice representing the 32-bit pixels in the framebuffer
    fb: &'static mut [u32],

    /// Useful information on the framebuffer often needed to properly plot pixels
    info: FramebufferInfo,

    /// The current foreground and background colors
    color: ColorCode,
}

impl Writer {
    fn new(fb: &'static mut [u32], info: FramebufferInfo) -> Self {
        Self {
            fb,
            info,
            color: ColorCode::default(),
        }
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

impl Write for Writer {
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

pub macro print {
    ($($arg:tt)*) => ($crate::graphics::_print(format_args!($($arg)*))),
}

pub macro println {
    () => ($crate::graphics::print!("\n")),
    ($($arg:tt)*) => ($crate::graphics::print!("{}\n", format_args!($($arg)*))),
}

pub fn init(fb: &Framebuffer) {
    let fb_info = FramebufferInfo::new(fb.width, fb.height, fb.pitch, fb.bpp);

    // SAFETY: `as_ptr` will never return `None`
    let fb_addr = unsafe { fb.address.as_ptr().unwrap_unchecked() };
    // SAFETY: `fb_addr` is non-null and aligned
    let fb_slice = unsafe { core::slice::from_raw_parts_mut(fb_addr.cast::<u32>(), fb.size()) };

    let writer = Writer::new(fb_slice, fb_info);

    WRITER.call_once(|| Mutex::new(writer));
}
