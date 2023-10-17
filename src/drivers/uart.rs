use core::fmt::{Arguments, Write};
use spin::{Mutex, Once};
use uart_16550::SerialPort;

pub static SERIAL1: Once<Mutex<SerialPort>> = Once::new();

#[doc(hidden)]
pub fn _print(args: Arguments<'_>) {
    SERIAL1.get().unwrap().lock().write_fmt(args);
}

pub macro serial_print($($arg:tt)*) {
    $crate::drivers::uart::_print(format_args!($($arg)*));
}

pub macro serial_println {
    () => ($crate::graphics::print!("\n")),
    ($($arg:tt)*) => ($crate::drivers::uart::print!("{}\n", format_args!($($arg)*))),
}

/// Initialize the UART serial port.
pub fn init() {
    // SAFETY: the base address passed points to a serial port.
    let mut sp = unsafe { SerialPort::new(0x3F8) };
    sp.init();

    SERIAL1.call_once(|| Mutex::new(sp));
}
