#![no_std]
#![no_main]

use lib::sched_yield;

#[macro_use]
extern crate lib;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("Hello world from user mode program!");
    sched_yield();
    println!("Array: {:#?}", [1, 2, 3, 4, 5]);
    sched_yield();
    println!("Hello, world, {}! {}", "Rust", 2024);
    0
}
