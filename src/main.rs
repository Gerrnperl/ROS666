#![no_std]
#![no_main]

mod language_item;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));

// #[unsafe(no_mangle)]
// pub extern "C" fn _start() -> ! {
//     loop {}
// }
