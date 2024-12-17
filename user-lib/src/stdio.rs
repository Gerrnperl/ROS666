use core::fmt::Write;

use crate::{read, write};

const STDIN: usize = 0;

pub fn getchar() -> u8 {
    let mut c = [0u8; 1];
    read(STDIN, &mut c);
    c[0]
}

const STDOUT: usize = 1;
struct Stdout {}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let ret = write(STDOUT, s.as_bytes());
        if ret < 0 {
            return Err(core::fmt::Error);
        }
        Ok(())
    }
}

pub fn print(args: core::fmt::Arguments) {
    if let Err(e) = (Stdout {}.write_fmt(args)) {
        panic!("Printing to stdout failed: {:?}", e);
    }
}

#[macro_export]
macro_rules! print {
    ($fmt:expr) => {
        $crate::stdio::print(format_args!($fmt))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::stdio::print(format_args!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($fmt:expr) => {
        $crate::print!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::print!(concat!($fmt, "\n"), $($arg)*)
    };
}
