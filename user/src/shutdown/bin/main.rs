#![no_std]
#![no_main]

use lib::shutdown;

#[unsafe(no_mangle)]
pub fn main() -> ! {
    shutdown();
}
