use limine::Framebuffer;
use spin::{Mutex, Once};

#[derive(Debug)]
struct FramebufferInfo {
    byte_len: usize,
    pitch: u64,
}

impl FramebufferInfo {
    fn new(byte_len: usize, pitch: u64) -> Self {
        Self { byte_len, pitch }
    }
}

#[derive(Debug)]
struct Writer<'a> {
    fb_slice: &'a mut [u8],
    fb_info: FramebufferInfo,
}

impl<'a> Writer<'a> {
    fn new(fb_slice: &'a mut [u8], fb_info: FramebufferInfo) -> Self {
        Self { fb_slice, fb_info }
    }
}

static WRITER: Once<Mutex<Writer>> = Once::new();

pub fn init(fb: &Framebuffer) {
    let fb_info = FramebufferInfo::new(fb.size(), fb.pitch);

    // SAFETY: `as_ptr` will never return `None`
    let fb_addr = unsafe { fb.address.as_ptr().unwrap_unchecked() };
    // SAFETY: `fb_addr` is non-null and aligned
    let fb_slice = unsafe { core::slice::from_raw_parts_mut(fb_addr, fb_info.byte_len) };

    let writer = Writer::new(fb_slice, fb_info);

    WRITER.call_once(|| Mutex::new(writer));
}
