#![no_std]
#![no_main]
#![feature(linkage)]
#![feature(alloc_error_handler)]

pub mod language_item;
pub mod sched;
pub mod stdio;
pub mod sys;
pub mod syscall;
pub mod unistd;

use heap_allocator::init_heap;
pub use language_item::*;
pub use sched::*;
pub use stdio::*;
pub use syscall::*;
pub use unistd::*;

mod heap_allocator;

use core::arch::{asm, global_asm};

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    init_heap();
    exit(main());
}

#[linkage = "weak"]
#[unsafe(no_mangle)]
fn main() -> i32 {
    unreachable!("This main should be replaced by user code");
}

/// 清空 BSS 段
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
