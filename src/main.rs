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
    printkln!("Hello, {}!", "World");
    printkln!("Hello"); // => "Hello"
    printkln!("Hello, {}!", "world"); // => "Hello, world!"
    printkln!("The number is {}", 1); // => "The number is 1"
    printkln!("{:?}", (3, 4)); // => "(3, 4)"
    printkln!("{value}", value = 4); // => "4"
    printkln!("{} {}", 1, 2); // => "1 2"
    printkln!("{:04}", 42); // => "0042" with leading zeros
    printkln!("{:#?}", (100, 200));

    // sbi::sbi_shutdown(false);
    loop {}
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
