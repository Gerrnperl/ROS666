//! 标准输入输出

use core::fmt::Write;

use crate::{read, write};

const STDIN: usize = 0;

/// 从标准输入读取一个字符
///
/// ## 返回值
/// 返回读取到的字符
pub fn getchar() -> u8 {
    let mut c = [0u8; 1];
    read(STDIN, &mut c);
    c[0]
}

const STDOUT: usize = 1;
struct Stdout {}

/// 实现 `Write` trait 用于 `Stdout`。
impl Write for Stdout {
    /// * `write_str` - 将字符串写入标准输出。
    /// # 参数
    /// * `s` - 要写入的字符串切片。
    /// # 返回值
    /// 如果写入成功，返回 `Ok(())`，否则返回 `Err(core::fmt::Error)`。
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let ret = write(STDOUT, s.as_bytes());
        if ret < 0 {
            return Err(core::fmt::Error);
        }
        Ok(())
    }
}

/// 打印格式化字符串
///
/// ## 参数
/// - `args`: 格式化字符串的参数
/// ## 返回值
/// 返回格式化结果
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
