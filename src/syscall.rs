use common::syscall::{
    Syscall, SyscallArgs, SyscallRet,
    time::{TimeVal, TimeZone},
};

use crate::{
    mm::page_table::translated_byte_buffer,
    printk,
    task::{self, manager::TaskManager},
    timer::{get_time, get_time_us},
};

pub fn syscall(call: Syscall, args: SyscallArgs) -> SyscallRet {
    match call {
        Syscall::Read => todo!(),
        Syscall::Write => sys_write(args[0], args[1] as *const u8, args[2]),
        Syscall::Exit => {
            sys_exit(args[0]);
            0
        }
        Syscall::SchedYield => sys_yield(),
        Syscall::GetTimeOfDay => sys_get_time_of_day(args[0] as *mut _, args[1] as *mut _),
        #[allow(
            unreachable_patterns,
            reason = "we may receive syscall numbers not defined in the enum"
        )]
        _ => panic!("Unsupported syscall: {}", call),
    }
}

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buffer: *const u8, len: usize) -> SyscallRet {
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(TaskManager::current_user_token(), buffer, len);
            for buffer in buffers {
                let s = core::str::from_utf8(buffer).unwrap();
                printk!("{}", s);
            }
            len as SyscallRet
        }
        _ => panic!("Unsupported file descriptor: {}", fd),
    }
}

pub fn sys_exit(code: usize) {
    printk!("Process exited with code {}\n", code);
    task::manager::TaskManager::replace_to_next();
}

pub fn sys_yield() -> SyscallRet {
    task::manager::TaskManager::cycle_to_next();
    0
}

pub fn sys_get_time_of_day(ts: *mut TimeVal, tz: *mut TimeZone) -> SyscallRet {
    let ts = unsafe { ts.as_mut().unwrap() };
    let tz = unsafe { tz.as_mut().unwrap() };
    let time = get_time_us();
    ts.tv_sec = time / 1_000_000;
    ts.tv_usec = time % 1_000_000;
    tz.tz_minuteswest = 0;
    tz.tz_dsttime = 0;
    0
}
