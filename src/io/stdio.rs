use core::fmt::Write;

use crate::sbi::{self, sbi_console_getchar};

struct Stdout {}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        sbi::sbi_console_put(s).map_err(|_| core::fmt::Error)?;
        // for c in s.chars() {
        //     let _ = sbi::sbi_console_putchar(c as usize);
        // }
        Ok(())
    }
}

pub fn read_str(buf: &mut [u8], len: usize) {
    for i in 0..len {
        match sbi_console_getchar() {
            Ok(c) => buf[i] = c,
            Err(_) => break,
        }
    }
}

pub fn printk(args: core::fmt::Arguments) {
    if let Err(e) = (Stdout {}.write_fmt(args)) {
        panic!("Printing to stdout failed: {:?}", e);
    }
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
