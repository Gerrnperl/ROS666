#![no_std]
#![no_main]

use lib::println;

#[unsafe(no_mangle)]
pub fn main(_argc: usize, argv: &[&str]) -> i32 {
    println!("{}", argv.join(" "));
    0
}
