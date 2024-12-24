#![no_std]
#![no_main]

use lib::{
    fcntl::{OpenFlags, close, openat},
    read, write,
};
extern crate alloc;
#[macro_use]
extern crate lib;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let content = "Hello fs";
    let file = "test.txt";
    let fd = openat(file, OpenFlags::CREATE | OpenFlags::WRITEONLY);
    println!("fd: {}", fd);
    assert!(fd >= 0, "Failed to open file");
    let fd = fd as usize;
    write(fd, content.as_bytes());
    close(fd as i32);

    let fd = openat(file, OpenFlags::READONLY);
    assert!(fd >= 0, "Failed to open file");
    let fd = fd as usize;
    let mut buf = [0u8; 32];
    let size = read(fd, &mut buf);
    close(fd as i32);

    let content = core::str::from_utf8(&buf[..size as usize]).unwrap();
    println!("Read content: {}", content);
    0
}
