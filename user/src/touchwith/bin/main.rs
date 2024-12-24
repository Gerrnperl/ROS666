#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;
use lib::fcntl::{OpenFlags, openat};
use lib::{fcntl::close, write};

#[unsafe(no_mangle)]
pub fn main(_argc: usize, argv: &[&str]) -> i32 {
    let mut file = argv[0].to_string();
    file.push('\0');
    let file = file.as_str();
    let fd = openat(file, OpenFlags::CREATE | OpenFlags::WRITEONLY) as usize;
    let content = argv[1..].join(" ");
    write(fd, content.as_bytes());
    close(fd as i32);
    0
}
