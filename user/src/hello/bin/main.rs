//! # Hello World

#![no_std]
#![no_main]

use lib::{TimeVal, sched_yield, sys::time::get_time_of_day};

#[macro_use]
extern crate lib;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    // 打印欢迎信息
    println!("Hello world from user mode program!");
    // 调用调度让出函数
    sched_yield();
    // 打印数组
    println!("Array: {:#?}", [1, 2, 3, 4, 5]);
    // 再次调用调度让出函数
    sched_yield();
    let mut a: usize = 0;
    const CLOCK_FREQ: usize = 125000;
    // 循环打印信息
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
    // 获取当前时间
    get_time_of_day(&mut ts, None);
    // 打印时间信息
    println!("Time: {}.{:06}", ts.tv_sec, ts.tv_usec);
    0
}
