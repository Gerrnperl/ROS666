//! 这个模块包含语言项的实现。
//!
//! 主要功能包括：
//! - 自定义 panic 处理函数。

use core::panic::PanicInfo;

use crate::{exit, print, println};

/// 自定义 panic 处理函数
///
/// ## 参数
/// - `info`: 包含 panic 信息的 `PanicInfo` 结构体
/// ## 返回值
/// 该函数不会返回，直接调用 `exit(1)` 退出程序
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let message = info.message();
    print!("Panicked at '{:?}'", message);
    if let Some(location) = info.location() {
        let file = location.file();
        let line = location.line();
        let column = location.column();
        println!(", {}:{}:{}", file, line, column);
    }
    exit(1);
}
