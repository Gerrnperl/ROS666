//! # initproc

#![no_std]
#![no_main]

use lib::{execve, fork, sched_yield, sys::wait::wait};

#[macro_use]
extern crate lib;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    // 如果 fork 返回 0，表示这是子进程
    if fork() == 0 {
        // 在子进程中执行 "sh" 程序
        execve("sh", None);
    } else {
        // 父进程进入循环
        loop {
            let mut exit_code = 0;
            // 等待子进程退出，并获取其退出码
            let pid = wait(&mut exit_code);
            if pid == -1 {
                // 如果没有子进程退出，则让出 CPU
                sched_yield();
                continue;
            }
            // 打印子进程的退出信息
            println!("Process {} exited with code {}", pid, exit_code);
        }
    }
    0
}
