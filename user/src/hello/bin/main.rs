#![no_std]
#![no_main]

use lib::{TimeVal, sched_yield, sys::time::get_time_of_day};

#[macro_use]
extern crate lib;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("Hello world from user mode program!");
    sched_yield();
    println!("Array: {:#?}", [1, 2, 3, 4, 5]);
    sched_yield();
    let mut a: usize = 0;
    const CLOCK_FREQ: usize = 125000;
    for i in 0..(125000 * 10) {
        if i % CLOCK_FREQ == 0 {
            println!("Hello, world! {}", a);
            a += 1;
        }
    }
    let mut ts = TimeVal {
        tv_sec: 0,
        tv_usec: 0,
    };
    get_time_of_day(&mut ts, None);
    println!("Time: {}.{:06}", ts.tv_sec, ts.tv_usec);
    0
}
