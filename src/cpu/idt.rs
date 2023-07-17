pub fn hlt() -> ! {
    loop {
        unsafe { x86::halt() }
    }
}
