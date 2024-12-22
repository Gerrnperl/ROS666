#![no_std]
#![no_main]

use lib::{execve, fork};

#[unsafe(no_mangle)]
pub fn main() {
    if fork() == 0 {
        execve("hello\0");
    } else {
        execve("bye\0");
    }
}
