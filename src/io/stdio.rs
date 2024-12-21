//! 标准输入输出模块

use core::fmt::Write;

/// 导入 `sbi` 模块及其 `sbi_console_getchar` 函数。
/// 导入 `task::manager::TaskManager` 模块。
use crate::{
    sbi::{self, sbi_console_getchar},
    task::manager::TaskManager,
};

/// 标准输出结构体
struct Stdout {}

/// 为 `Stdout` 实现 `Write` trait
impl Write for Stdout {
    /// 实现 `write_str` 方法，将字符串写入标准输出
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // 使用 `sbi_console_put` 将字符串输出到控制台
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
    // 初始化索引
    let mut i = 0;
    // 循环读取字符
    loop {
        match sbi_console_getchar() {
            // 如果读取到空字符，则切换到下一个任务
            Ok(0) => {
                TaskManager::cycle_to_next();
                continue;
            }
            // 如果读取到有效字符，则存储到缓冲区
            Ok(c) => {
                buf[i] = c;
                i += 1;
                // 如果读取到的字符数达到指定长度，则退出循环
                if i == len {
                    break;
                }
            }
            // 如果读取失败，则退出循环
            Err(_) => break,
        }
    }
}

/// 打印格式化的字符串到标准输出
pub fn printk(args: core::fmt::Arguments) {
    // 尝试将格式化的字符串写入标准输出
    if let Err(e) = (Stdout {}.write_fmt(args)) {
        // 如果写入失败，则触发恐慌并打印错误信息
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
