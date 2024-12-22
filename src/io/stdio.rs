//! 终端输入输出模块

use core::fmt::Write;

use crate::{
    sbi::{self, sbi_console_getchar},
    task::manager::TaskManager,
};

struct Stdout {}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        sbi::sbi_console_put(s).map_err(|_| core::fmt::Error)?;
        // 遍历字符串的每个字符并输出到控制台
        // for c in s.chars() {
        //     let _ = sbi::sbi_console_putchar(c as usize);
        // }
        Ok(())
    }
}

/// 从标准输入读取字符串
///
/// ## 参数
/// * `buf` - 存储读取到的字符串的缓冲区
/// * `len` - 要读取的字符串的长度
pub fn read_str(buf: &mut [u8], len: usize) {
    let mut i = 0;
    loop {
        match sbi_console_getchar() {
            Ok(0) => {
                TaskManager::cycle_to_next();
                continue;
            }
            Ok(c) => {
                buf[i] = c;
                i += 1;
                if i == len {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

/// 打印格式化的字符串到标准输出
pub fn printk(args: core::fmt::Arguments) {
    if let Err(e) = (Stdout {}.write_fmt(args)) {
        panic!("Printing to stdout failed: {:?}", e);
    }
}

/// 打印格式化字符串到标准输出的宏
///
/// ## 示例
/// ```rust
/// printk!("Hello, world!");
/// printk!("Hello, {}!", "world");
/// ```
#[macro_export]
macro_rules! printk {
    ($fmt:expr) => {
        $crate::io::stdio::printk(format_args!($fmt))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::io::stdio::printk(format_args!($fmt, $($arg)*))
    };
}

/// 打印格式化字符串并换行到标准输出的宏
///
/// ## 示例
/// ```rust
/// printkln!();
/// printkln!("Hello, world!");
/// printkln!("Hello, {}!", "world");
/// ```
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
