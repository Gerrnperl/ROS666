#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("Hello world from user mode program!");
    0
}
