use core::fmt::{Arguments, Write};
use spin::{Lazy, Mutex};
use uart_16550::SerialPort;

pub static SERIAL1: Lazy<Mutex<SerialPort>> = Lazy::new(|| {
    let mut sp = unsafe { SerialPort::new(0x3F8) };
    sp.init();
    Mutex::new(sp)
});

#[doc(hidden)]
pub fn _print(args: Arguments<'_>) {
    SERIAL1.lock().write_fmt(args);
}

pub macro serial_print($($arg:tt)*) {
    $crate::drivers::uart::_print(format_args!($($arg)*));
}

pub macro serial_println {
    () => ($crate::graphics::print!("\n")),
    ($($arg:tt)*) => ($crate::drivers::uart::print!("{}\n", format_args!($($arg)*))),
}
