#![no_std]
#![no_main]

mod io;
mod language_item;
mod sbi;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));

/// 内核入口函数
#[unsafe(no_mangle)]
pub extern "C" fn _kernel_entry() -> ! {
    clear_bss();
    startup_log();
    printkln!("Hello, {}!", "World");

    // sbi::sbi_shutdown(false);
    loop {}
}

fn startup_log() {
    unsafe extern "C" {
        static mut __text_start: u64;
        static mut __text_end: u64;
        static mut __rodata_start: u64;
        static mut __rodata_end: u64;
        static mut __data_start: u64;
        static mut __data_end: u64;
        static mut __bss_start: u64;
        static mut __bss_end: u64;
    }

    let text_start = unsafe { __text_start as usize };
    let text_end = unsafe { __text_end as usize };
    let rodata_start = unsafe { __rodata_start as usize };
    let rodata_end = unsafe { __rodata_end as usize };
    let data_start = unsafe { __data_start as usize };
    let data_end = unsafe { __data_end as usize };
    let bss_start = unsafe { __bss_start as usize };
    let bss_end = unsafe { __bss_end as usize };

    info!("[Kernel] Secion:");
    info!("[Kernel]  text   : [{:#x}, {:#x})", text_start, text_end);
    info!(
        "[Kernel]  rodata : [{:#x}, {:#x})",
        rodata_start, rodata_end
    );
    info!("[Kernel]  data   : [{:#x}, {:#x})", data_start, data_end);
    info!("[Kernel]  bss    : [{:#x}, {:#x})", bss_start, bss_end);
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
