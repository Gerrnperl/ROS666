use core::fmt::Write;

use crate::sbi;

struct Stdout {}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        sbi::sbi_console_put(s).map_err(|_| core::fmt::Error)?;
        Ok(())
    }
}

pub fn printk(args: core::fmt::Arguments) {
    Stdout {}.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! printk {
    ($fmt:expr) => {
        $crate::io::stdio::printk(format_args!($fmt))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::io::stdio::printk(format_args!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! printkln {
    () => {
        $crate::printk!("\n")
    };
    ($fmt:expr) => {
        $crate::printk!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::printk!(concat!($fmt, "\n"), $($arg)*)
    };
}
