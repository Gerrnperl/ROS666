#![no_std]
#![no_main]

use lib::{
    fcntl::{OpenFlags, openat},
    println, read,
};

#[unsafe(no_mangle)]
pub fn main(_argc: usize, _argv: &[&str]) -> i32 {
    let fd = openat("/\0", OpenFlags::READONLY) as usize;
    let mut buf = [0u8; 1024];
    let size = read(fd, &mut buf);
    let content = core::str::from_utf8(&buf[..size as usize]).unwrap();
    println!("{}", content);
    0
}
