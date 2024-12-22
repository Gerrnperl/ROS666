//! Rust 语言项

use core::panic::PanicInfo;

use crate::{exit, print, println};

/// Rust 语言项 - panic 处理函数
///
/// 当 panic 时会调用该函数
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
