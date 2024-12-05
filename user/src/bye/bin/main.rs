#![no_std]
#![no_main]

use lib::sched_yield;

#[macro_use]
extern crate lib;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let mut a: usize = 0;
    const CLOCK_FREQ: usize = 12500000;
    for i in 0..(12500000 * 10) {
        if i % CLOCK_FREQ == 0 {
            println!("Bye, world! {}", a);
            a += 1;
        }
    }
    sched_yield();
    println!("Bye, world! {}", a);
    sched_yield();
    panic!("Bye!");
    0
}
