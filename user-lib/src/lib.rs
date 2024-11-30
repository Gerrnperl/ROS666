#![no_std]
#![no_main]
#![feature(linkage)]

pub mod language_item;
pub mod stdio;
pub mod syscall;
pub mod unistd;

pub use language_item::*;
pub use stdio::*;
pub use syscall::*;
pub use unistd::*;

use core::arch::{asm, global_asm};

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    clear_bss();
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
        static mut __bss_start: u64;
        static mut __bss_end: u64;
    }

    let bss_start = unsafe { __bss_start as usize };
    let bss_end = unsafe { __bss_end as usize };

    for i in bss_start..bss_end {
        unsafe {
            core::ptr::write_volatile(i as *mut u8, 0);
        }
    }
}
