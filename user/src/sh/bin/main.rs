//! # Shell

#![no_std]
#![no_main]

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use lib::{execve, fork, getchar, sys::wait::waitpid};

extern crate alloc;
#[macro_use]
extern crate lib;

const LF: u8 = 10; // 换行符
const CR: u8 = 13; // 回车符
const BS: u8 = 8; // 退格符
const DEL: u8 = 127; // 删除符

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    loop {
        print_prompt(); // 打印提示符
        let command = get_command(true); // 获取用户输入的命令
        if command.len() == 0 {
            continue; // 如果命令为空，继续循环
        }
        let (mut program, args) = parse_command(command.as_str());
        program.push('\0');
        let program = program.as_str();
        let args_vec: Option<Vec<&str>> = args
            .as_ref()
            .map(|args| args.iter().map(|arg| arg.as_str()).collect::<Vec<&str>>());
        let args_ptr = args_vec.as_ref().map(|args| args.as_slice());
        let pid = fork(); // 创建子进程
        if pid == 0 {
            let exit_code = execve(program, args_ptr); // 在子进程中执行命令
            if exit_code != 0 {
                println!("Failed to execute command: {}", command); // 命令执行失败
                return -4;
            }
            unreachable!(); // 不应该到达这里
        } else {
            let mut exit_code = 0;
            let exit_pid = waitpid(pid as usize, &mut exit_code); // 等待子进程结束
            assert_eq!(exit_pid, pid); // 确保等待的进程是刚刚创建的子进程
            println!("Process {} exited with code {}", pid, exit_code); // 打印子进程的退出状态
        }
    }
}

fn print_prompt() {
    print!("> "); // 打印提示符
}

fn get_command(echo: bool) -> String {
    let mut buffer = String::new(); // 创建一个新的字符串缓冲区
    loop {
        let c = getchar(); // 获取一个字符
        match c {
            CR | LF => {
                if echo {
                    println!(); // 如果需要回显，打印换行
                }
                break; // 结束输入
            }
            BS | DEL => {
                if buffer.len() > 0 {
                    buffer.pop(); // 删除缓冲区中的最后一个字符
                    if echo {
                        print!("\u{8} \u{8}"); // 如果需要回显，删除最后一个字符
                    }
                }
            }
            _ => {
                buffer.push(c as char); // 将字符添加到缓冲区
                if echo {
                    print!("{}", c as char); // 如果需要回显，打印字符
                }
            }
        }
    }
    buffer // 返回缓冲区中的字符串
}

fn parse_command(command: &str) -> (String, Option<Vec<String>>) {
    let mut parts = command.split_whitespace(); // 使用空白字符分割命令
    let program = parts.next().unwrap().to_string(); // 第一个部分是程序名
    let args = parts.collect::<Vec<&str>>(); // 剩余部分是参数

    // add \0 to the end of each argument
    let args_with_null: Vec<String> = args
        .iter()
        .map(|&arg| {
            let mut arg = arg.to_string();
            arg.push('\0');
            arg
        })
        .collect();

    (program, Some(args_with_null))
}
