#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::println::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\r\n"));
    ($($arg:tt)*) => ($crate::print!("{}\r\n", format_args!($($arg)*)));
}

pub fn _print(args: core::fmt::Arguments) {
    use crate::uart::UART;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        use core::fmt::Write;
        UART.lock().write_fmt(args).unwrap();
    });
}
