//! # Bye

#![no_std]
#![no_main]

use lib::sched_yield;

#[macro_use]
extern crate lib;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let mut a: usize = 0;
    const CLOCK_FREQ: usize = 1250000;
    // 循环 1250000 * 10 次
    for i in 0..(1250000 * 10) {
        // 每当 i 是 CLOCK_FREQ 的倍数时，打印 "Bye, world!" 和 a 的值
        if i % CLOCK_FREQ == 0 {
            println!("Bye, world! {}", a);
            a += 1;
        }
    }
    // 调用 sched_yield 函数
    sched_yield();
    // 再次打印 "Bye, world!" 和 a 的值
    println!("Bye, world! {}", a);
    // 再次调用 sched_yield 函数
    sched_yield();
    // 触发 panic，打印 "Bye!"
    panic!("Bye!");
    0
}
