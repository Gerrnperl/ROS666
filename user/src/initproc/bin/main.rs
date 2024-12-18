#![no_std]
#![no_main]

use lib::{execve, fork, sched_yield, sys::wait::wait};

#[macro_use]
extern crate lib;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    if fork() == 0 {
        execve("sh");
    } else {
        loop {
            let mut exit_code = 0;
            let pid = wait(&mut exit_code);
            if pid == -1 {
                sched_yield();
                continue;
            }
            println!("Process {} exited with code {}", pid, exit_code);
        }
    }
    0
}
