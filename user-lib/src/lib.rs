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

use alloc::vec::Vec;
use heap_allocator::init_heap;
pub use sched::*;
pub use stdio::*;
pub use syscall::*;
pub use unistd::*;

mod heap_allocator;

extern crate alloc;

/// 程序入口点
///
/// 该函数是程序的入口点，负责初始化堆并调用用户定义的 `main` 函数。
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
// first arg is the return value of sys_execve, ignore it
pub extern "C" fn _start(_: usize, argc: usize, argv: usize) -> ! {
    init_heap(); // 初始化堆
    let mut v: Vec<&'static str> = Vec::new();
    println!("argc: {}", argc);
    println!("argv: {}", argv);
    let mut argv_ptr = argv;
    for _ in 0..argc {
        let str_start = argv_ptr as *const u8;
        let mut len = 0;
        unsafe {
            while *((str_start as usize + len) as *const u8) != 0 {
                len += 1;
            }
        }
        let slice = unsafe { core::slice::from_raw_parts(str_start, len) };
        let str_slice = core::str::from_utf8(slice).unwrap();
        v.push(str_slice);
        argv_ptr += len + 1;
    }
    exit(main(argc, v.as_slice()));
}

/// 用户定义的主函数
///
/// 该函数是一个弱链接，应该被用户代码中的 `main` 函数替换。
#[linkage = "weak"]
#[unsafe(no_mangle)]
fn main(_argc: usize, _argv: &[&str]) -> i32 {
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
