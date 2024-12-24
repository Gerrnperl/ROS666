//! # Hello World

#![no_std]
#![no_main]

use lib::{TimeVal, sched_yield, sleep, sys::time::get_time_of_day};

#[macro_use]
extern crate lib;

#[unsafe(no_mangle)]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    // 打印欢迎信息
    println!("Hello world from user mode program!");

    // 打印参数
    println!("Arguments: argc = {}, argv = {:?}", argc, argv);

    // 调用调度让出函数
    sched_yield();
    // 打印数组
    println!("Array: {:#?}", [1, 2, 3, 4, 5]);
    for i in 0..10 {
        sleep(1);
        println!("[Hello] Tick {}", i);
    }
    let mut ts = TimeVal {
        tv_sec: 0,
        tv_usec: 0,
    };
    // 获取当前时间
    get_time_of_day(&mut ts, None);
    // 打印时间信息
    println!("[Hello] Time: {}.{:06}", ts.tv_sec, ts.tv_usec);
    0
}
