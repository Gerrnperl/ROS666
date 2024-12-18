#![no_std]
#![no_main]

use alloc::string::String;
use lib::{execve, fork, getchar, sys::wait::waitpid};

extern crate alloc;
#[macro_use]
extern crate lib;

const LF: u8 = 10;
const CR: u8 = 13;
const BS: u8 = 8;
const DEL: u8 = 127;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    loop {
        print_prompt();
        let command = get_command(true);
        if command.len() == 0 {
            continue;
        }
        let pid = fork();
        if pid == 0 {
            let exit_code = execve(command.as_str());
            if exit_code != 0 {
                println!("Failed to execute command: {}", command);
                return -4;
            }
            unreachable!();
        } else {
            let mut exit_code = 0;
            let exit_pid = waitpid(pid as usize, &mut exit_code);
            assert_eq!(exit_pid, pid);
            println!("Process {} exited with code {}", pid, exit_code);
        }
    }
}

fn print_prompt() {
    print!("> ");
}

fn get_command(echo: bool) -> String {
    let mut buffer = String::new();
    loop {
        let c = getchar();
        match c {
            CR | LF => {
                if echo {
                    println!();
                }
                break;
            }
            BS | DEL => {
                if buffer.len() > 0 {
                    buffer.pop();
                    if echo {
                        print!("\u{8} \u{8}");
                    }
                }
            }
            _ => {
                buffer.push(c as char);
                if echo {
                    print!("{}", c as char);
                }
            }
        }
    }
    buffer
}
