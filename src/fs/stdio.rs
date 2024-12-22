use crate::{io::stdio::read_str, mm::page_table::UserBuffer, printk};

use super::File;

pub struct Stdin;
pub struct Stdout;

impl File for Stdin {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    fn read(&self, user_buf: UserBuffer) -> usize {
        let mut len = 0;
        for buffer in user_buf.buffers {
            read_str(buffer, buffer.len());
            len += buffer.len();
        }
        len
    }
    fn write(&self, _user_buf: UserBuffer) -> usize {
        panic!("Stdin is not writable");
    }
}

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, _user_buf: UserBuffer) -> usize {
        panic!("Cannot read from stdout!");
    }
    fn write(&self, user_buf: UserBuffer) -> usize {
        let mut len = 0;
        for buffer in user_buf.buffers {
            let s = core::str::from_utf8(buffer).unwrap();
            printk!("{}", s);
            len += s.len();
        }
        len
    }
}
