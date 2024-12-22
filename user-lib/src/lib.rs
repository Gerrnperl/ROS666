//! 这个模块包含用户库的主要功能。
//!
//! 主要功能包括：
//! - 清空 BSS 段。
//! - 提供系统调用接口。
//! - 提供堆分配器。
//! - 提供与时间相关的系统调用接口。
//! - 提供与等待相关的系统调用接口。

#![no_std]
#![no_main]
#![feature(linkage)]
#![feature(alloc_error_handler)]

pub mod fcntl;
/// 语言项模块
pub mod language_item;
/// 调度模块
pub mod sched;
/// 标准输入输出模块
pub mod stdio;
/// 系统模块
pub mod sys;
/// 系统调用模块
pub mod syscall;
/// Unix 标准模块
pub mod unistd;

/// 使用堆分配器初始化函数
use heap_allocator::init_heap;
/// 使用语言项模块中的所有内容
pub use language_item::*;
/// 使用调度模块中的所有内容
pub use sched::*;
/// 使用标准输入输出模块中的所有内容
pub use stdio::*;
/// 使用系统调用模块中的所有内容
pub use syscall::*;
/// 使用 Unix 标准模块中的所有内容
pub use unistd::*;

/// 堆分配器模块
mod heap_allocator;

/// 使用核心架构中的汇编和全局汇编
use core::arch::{asm, global_asm};

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
