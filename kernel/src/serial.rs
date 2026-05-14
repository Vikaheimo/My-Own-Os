use spin::{Mutex, Once};
use uart_16550::SerialPort;

pub static SERIAL1: Once<Mutex<SerialPort>> = Once::new();

pub fn init() {
    SERIAL1.call_once(|| {
        let mut port = unsafe { SerialPort::new(0x3F8) };
        port.init();
        Mutex::new(port)
    });
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1
            .get()
            .unwrap()
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
