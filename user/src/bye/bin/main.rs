#![no_std]
#![no_main]

use lib::sched_yield;

#[macro_use]
extern crate lib;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let mut a: usize = 0;
    const CLOCK_FREQ: usize = 12500000;
    let ten_seconds = 5 * CLOCK_FREQ;
    for i in 0..ten_seconds {}
    sched_yield();
    println!("Bye, world! {}", a);
    sched_yield();
    panic!("Bye!");
    0
}
