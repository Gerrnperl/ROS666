use alloc::sync::Arc;
use common::syscall::{
    Syscall, SyscallArgs, SyscallRet,
    time::{TimeVal, TimeZone},
};

use crate::{
    app_loader::load_app_data_by_name,
    io::stdio::read_str,
    mm::page_table::{
        get_mut_translated_byte_slices, get_translated_byte_slices, get_translated_refmut,
        get_translated_string,
    },
    printk, printkln,
    task::{self, manager::TaskManager, pid, processor::Processor, task::ProcessStatus},
    timer::{get_time, get_time_us},
    trap::context::Riscv64RegAlias,
};

pub fn syscall(call: Syscall, args: SyscallArgs) -> SyscallRet {
    match call {
        Syscall::Read => sys_read(args[0], args[1] as *mut u8, args[2]),
        Syscall::Write => sys_write(args[0], args[1] as *const u8, args[2]),
        Syscall::Exit => {
            sys_exit(args[0] as i32);
            0
        }
        Syscall::SchedYield => sys_yield(),
        Syscall::GetTimeOfDay => sys_get_time_of_day(args[0] as *mut _, args[1] as *mut _),
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

pub fn sys_read(fd: usize, buffer: *mut u8, len: usize) -> SyscallRet {
    match fd {
        FD_STDIN => {
            let buffers = get_mut_translated_byte_slices(
                Processor::current_user_token().unwrap(),
                buffer,
                len,
            );
            for buffer in buffers {
                read_str(buffer, buffer.len());
            }
            len as SyscallRet
        }
        _ => panic!("Unsupported file descriptor: {}", fd),
    }
}

pub fn sys_write(fd: usize, buffer: *const u8, len: usize) -> SyscallRet {
    match fd {
        FD_STDOUT => {
            let buffers =
                get_translated_byte_slices(Processor::current_user_token().unwrap(), buffer, len);
            for buffer in buffers {
                let s = core::str::from_utf8(buffer).unwrap();
                printk!("{}", s);
            }
            len as SyscallRet
        }
        _ => panic!("Unsupported file descriptor: {}", fd),
    }
}

pub fn sys_exit(code: i32) {
    printk!("Process exited with code {}\n", code);
    task::manager::TaskManager::replace_to_next(code);
}

pub fn sys_yield() -> SyscallRet {
    task::manager::TaskManager::cycle_to_next();
    0
}

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

pub fn sys_clone() -> SyscallRet {
    let current = Processor::get_current().unwrap();
    let new_task = current.fork();
    let new_task_pid = new_task.inner_borrow().pid.0;
    let ctx = new_task.inner_borrow().get_trap_cx();
    *ctx.a(0) = 0; // return 0 for the child process
    TaskManager::put_task(new_task);
    new_task_pid as SyscallRet
}

pub fn sys_execve(path: *const u8) -> SyscallRet {
    let path = get_translated_string(Processor::current_user_token().unwrap(), path as *const u8);
    if let Some(data) = load_app_data_by_name(path.as_str()) {
        let current = Processor::get_current().unwrap();
        current.exec(data);
        0
    } else {
        printkln!("Failed to load app data: {}\n", path);
        -1
    }
}

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
