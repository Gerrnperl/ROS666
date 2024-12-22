//! 用户程序库
//!
//! 用于在用户程序中调用系统功能。
//!
//! 在无`std`的情况下，用户程序需要调用系统功能，例如`println!`、`exit`等，此时用户程序需要调用系统库。
//!
//! 此库提供了系统功能的实现，用户程序以此作为依赖，以调用系统功能。

#![no_std]
#![no_main]
#![feature(linkage)]
#![feature(alloc_error_handler)]

pub mod fcntl;
pub mod language_item;
pub mod sched;
pub mod stdio;
pub mod sys;
pub mod syscall;
pub mod unistd;

use heap_allocator::init_heap;
pub use sched::*;
pub use stdio::*;
pub use syscall::*;
pub use unistd::*;

mod heap_allocator;

/// 程序入口点
///
/// 该函数是程序的入口点，负责初始化堆并调用用户定义的 `main` 函数。
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    init_heap(); // 初始化堆
    exit(main()); // 调用用户定义的 `main` 函数并退出
}

/// 用户定义的主函数
///
/// 该函数是一个弱链接，应该被用户代码中的 `main` 函数替换。
#[linkage = "weak"]
#[unsafe(no_mangle)]
fn main() -> i32 {
    unreachable!("This main should be replaced by user code"); // 该 `main` 函数应该被用户代码替换
}

/// 清空 BSS 段
///
/// 该函数将 BSS 段的所有字节设置为 0。
#[allow(dead_code)]
fn clear_bss() {
    unsafe extern "C" {
        fn __bss_start();
        fn __bss_end();
    }
    for i in __bss_start as usize..__bss_end as usize {
        unsafe {
            core::ptr::write_volatile(i as *mut u8, 0);
        }
    }
}

/// 关闭系统
pub fn shutdown() -> ! {
    sys_shutdown();
}
