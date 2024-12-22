//! 系统调用实现
use alloc::sync::Arc;
use common::syscall::{
    OpenFlags, Syscall, SyscallArgs, SyscallRet,
    time::{TimeVal, TimeZone},
};

use crate::{
    fs::inode::open_file,
    mm::page_table::{
        UserBuffer, get_mut_translated_byte_slices, get_translated_byte_slices,
        get_translated_refmut, get_translated_string,
    },
    printkln,
    task::{self, manager::TaskManager, processor::Processor, task::ProcessStatus},
    timer::get_time_us,
    trap::context::Riscv64RegAlias,
    warn,
};

pub fn syscall(call: Syscall, args: SyscallArgs) -> SyscallRet {
    match call {
        Syscall::OpenAt => sys_openat(args[0] as *const u8, args[1] as usize),
        Syscall::Close => sys_close(args[0]),
        Syscall::Read => sys_read(args[0], args[1] as *mut u8, args[2]),
        Syscall::Write => sys_write(args[0], args[1] as *const u8, args[2]),
        Syscall::Exit => {
            sys_exit(args[0] as i32);
            0
        }
        Syscall::SchedYield => sys_yield(),
        Syscall::GetTimeOfDay => sys_get_time_of_day(args[0] as *mut _, args[1] as *mut _),
        Syscall::Shutdown => sys_shutdown(),
        Syscall::Clone => sys_clone(),
        Syscall::Execve => sys_execve(args[0] as *const u8),
        Syscall::Wait4 => sys_wait4(args[0] as isize, args[1] as *mut i32),
        #[allow(
            unreachable_patterns,
            reason = "we may receive syscall numbers not defined in the enum"
        )]
        _ => panic!("Unsupported syscall: {}", call),
    }
}

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

/// [Syscall::OpenAt]
pub fn sys_openat(path: *const u8, flags: usize) -> SyscallRet {
    let current = Processor::get_current().unwrap();
    let path = get_translated_string(Processor::current_user_token().unwrap(), path as *const u8);
    let inode = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap());
    if let Some(inode) = inode {
        let fd = current.inner_borrow_mut().alloc_fd();
        current.inner_borrow_mut().fd_table[fd] = Some(inode);
        fd as SyscallRet
    } else {
        -1
    }
}

/// [Syscall::Close]
pub fn sys_close(fd: usize) -> SyscallRet {
    let current = Processor::get_current().unwrap();
    let mut pcb = current.inner_borrow_mut();
    if fd >= pcb.fd_table.len() {
        return -1;
    }
    if pcb.fd_table[fd].is_none() {
        return -1;
    }
    pcb.fd_table[fd] = None;
    0
}

/// [Syscall::Read]
pub fn sys_read(fd: usize, buffer: *mut u8, len: usize) -> SyscallRet {
    let token = Processor::current_user_token().unwrap();
    let task = Processor::get_current().unwrap();
    let pcb = task.inner_borrow();
    if fd >= pcb.fd_table.len() {
        return -1;
    }
    if let Some(file) = &pcb.fd_table[fd] {
        let file = file.clone();
        if !(file.readable()) {
            return -1;
        }
        drop(pcb);
        file.read(UserBuffer::new(get_mut_translated_byte_slices(
            token, buffer, len,
        ))) as isize
    } else {
        -1
    }
}

/// [Syscall::Write]
pub fn sys_write(fd: usize, buffer: *const u8, len: usize) -> SyscallRet {
    let token = Processor::current_user_token().unwrap();
    let task = Processor::get_current().unwrap();
    let pcb = task.inner_borrow();
    if fd >= pcb.fd_table.len() {
        return -1;
    }
    if let Some(file) = &pcb.fd_table[fd] {
        let file = file.clone();
        if !(file.writable()) {
            return -1;
        }
        drop(pcb);
        file.write(UserBuffer::new(get_mut_translated_byte_slices(
            token, buffer, len,
        ))) as isize
    } else {
        -1
    }
}

/// [Syscall::Exit]
pub fn sys_exit(code: i32) {
    task::manager::TaskManager::replace_to_next(code);
}

/// [Syscall::SchedYield]
pub fn sys_yield() -> SyscallRet {
    task::manager::TaskManager::cycle_to_next();
    0
}

/// [Syscall::GetTimeOfDay]
pub fn sys_get_time_of_day(ts: *mut TimeVal, tz: *mut TimeZone) -> SyscallRet {
    let ts_buffer = get_translated_byte_slices(
        Processor::current_user_token().unwrap(),
        ts as *const u8,
        core::mem::size_of::<TimeVal>(),
    );
    let ts = unsafe { &mut *(ts_buffer[0].as_ptr() as *mut TimeVal) };
    let tz_buffer = get_translated_byte_slices(
        Processor::current_user_token().unwrap(),
        tz as *const u8,
        core::mem::size_of::<TimeZone>(),
    );
    let tz = unsafe { &mut *(tz_buffer[0].as_ptr() as *mut TimeZone) };
    let time = get_time_us();
    ts.tv_sec = time / 1_000_000;
    ts.tv_usec = time % 1_000_000;
    tz.tz_minuteswest = 0;
    tz.tz_dsttime = 0;
    0
}

/// [Syscall::Shutdown]
pub fn sys_shutdown() -> ! {
    warn!("System shutdown");
    crate::sbi::sbi_shutdown(false);
}

/// [Syscall::Clone]
pub fn sys_clone() -> SyscallRet {
    let current = Processor::get_current().unwrap();
    let new_task = current.fork();
    let new_task_pid = new_task.inner_borrow().pid.0;
    let ctx = new_task.inner_borrow().get_trap_cx();
    *ctx.a(0) = 0; // return 0 for the child process
    TaskManager::put_task(new_task);
    new_task_pid as SyscallRet
}

/// [Syscall::Execve]
pub fn sys_execve(path: *const u8) -> SyscallRet {
    let path = get_translated_string(Processor::current_user_token().unwrap(), path as *const u8);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::READONLY) {
        let data = app_inode.read_all();
        let data = data.as_slice();
        let current = Processor::get_current().unwrap();
        current.exec(data);
        0
    } else {
        printkln!("Failed to load app data: {}\n", path);
        -1
    }
}

/// [Syscall::Wait4]
pub fn sys_wait4(pid: isize, exit_code_ptr: *mut i32) -> SyscallRet {
    let task = Processor::get_current().unwrap();
    let mut pcb = task.inner_borrow_mut();

    let mut is_child_stopped = false;
    let mut child_index = None;

    for (index, child) in pcb.children.iter().enumerate() {
        let child_pcb = child.inner_borrow();
        if child_pcb.pid.0 == pid as usize || pid == -1 {
            child_index = Some(index);
            if child_pcb.status == ProcessStatus::Stopped {
                is_child_stopped = true;
            }
        }
    }

    if child_index.is_none() {
        return -1;
    }

    if is_child_stopped {
        let child = pcb.children.remove(child_index.unwrap());
        assert_eq!(Arc::strong_count(&child), 1);
        let child_pcb = child.inner_borrow();
        let child_exit_code = child_pcb.exit_code;
        let child_pid = child_pcb.pid.0;
        *get_translated_refmut(pcb.memory_set.token(), exit_code_ptr) = child_exit_code;

        child_pid as SyscallRet
    } else {
        -2
    }
}
