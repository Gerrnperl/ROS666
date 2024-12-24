#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;
use lib::{
    fcntl::{OpenFlags, openat},
    print, println, read,
};

#[unsafe(no_mangle)]
pub fn main(_argc: usize, argv: &[&str]) -> i32 {
    let mut file = argv[0].to_string();
    file.push('\0');
    let file = file.as_str();
    let fd = openat(file, OpenFlags::READONLY) as usize;
    loop {
        let mut buf = [0u8; 1024];
        let size = read(fd, &mut buf);
        if size == 0 {
            break;
        }
        let content = core::str::from_utf8(&buf[..size as usize]);
        if let Ok(content) = content {
            println!("{}", content);
        } else {
            for i in 0..size {
                print!("{:02x} ", buf[i as usize]);
            }
        }
    }
    0
}
