//! 中断处理模块

// 导入核心库中的 asm 和 usize 模块
use core::{arch::asm, usize};

// 导入公共库中的 Syscall 模块
use common::syscall::Syscall;
// 导入 RISC-V 架构相关的中断和寄存器模块
use riscv::{
    interrupt::{Exception, supervisor::Interrupt},
    register::{
        scause, stval,
        stvec::{self, TrapMode},
    },
};

use crate::{
    extern_global,
    mm::memory_set::TRAMPOLINE,
    syscall::syscall,
    task::{manager::TaskManager, processor::Processor, task::TRAP_CONTEXT},
    timer::set_next_timeout,
};

// 导入上级模块中的上下文相关定义
use super::context::Riscv64RegAlias;

/// 定义定时器中断间隔时间（微秒）
pub const TIMER_INTERVAL_USEC: usize = 10_000;

// 定义一个不进行名称修饰的函数
#[unsafe(no_mangle)]
// 用于处理用户态陷入
pub fn trap_handler() -> ! {
    // 设置用户态陷阱入口
    set_user_trap_entry();
    // 读取 scause 和 stval 寄存器的值
    let scause = scause::read().cause();
    let stval = stval::read();
    // 根据 scause 的值进行匹配处理
    match scause {
        // 用户态环境调用异常
        scause::Trap::Exception(const { Exception::UserEnvCall as usize }) => {
            let ctx = Processor::current_trap_cx().unwrap();
            ctx.sepc += 4; // 跳过 ecall 指令
            let ret = syscall(Syscall::from(*ctx.a(7)), [*ctx.a(0), *ctx.a(1), *ctx.a(2)]);
            // syscall 之后上下文可能会被修改，所以需要重新获取
            let ctx = Processor::current_trap_cx().unwrap();
            *ctx.a(0) = ret as usize; // 将系统调用返回值存入 a0 寄存器
        }
        // 存储错误或存储页错误异常
        scause::Trap::Exception(const { Exception::StoreFault as usize })
        | scause::Trap::Exception(const { Exception::StorePageFault as usize }) => {
            todo!("终止进程");
        }
        // 非法指令异常
        scause::Trap::Exception(const { Exception::IllegalInstruction as usize }) => {
            todo!("终止进程");
        }
        // 监督者定时器中断
        scause::Trap::Interrupt(const { Interrupt::SupervisorTimer as usize }) => {
            // todo!("Timer interrupt");
            // 设置下一个定时器中断
            set_next_timeout(TIMER_INTERVAL_USEC);
            // 切换到下一个任务
            TaskManager::cycle_to_next();
        }
        // 未处理的陷阱
        _ => {
            panic!("未处理的陷阱: {:?}, stval: {:#x}", scause, stval);
        }
    };
    // 返回到用户态
    trap_return();
}

/// 设置用户态陷阱入口函数
fn set_user_trap_entry() {
    unsafe {
        // 将 stvec 寄存器设置为 TRAMPOLINE 地址，陷阱模式为直接模式
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

// 定义一个不进行名称修饰的函数，用于返回到用户态
#[unsafe(no_mangle)]
/// 返回到用户态
pub fn trap_return() -> ! {
    // 设置用户态陷阱入口
    set_user_trap_entry();
    // 获取陷阱上下文指针
    let trap_cx_ptr = TRAP_CONTEXT;
    // 获取当前用户页表的 token
    let user_satp = Processor::current_user_token().unwrap();
    // 计算 __restore_trap 的虚拟地址
    let restore_va =
        extern_global!(__restore_trap) as usize - extern_global!(__save_trap) as usize + TRAMPOLINE;
    unsafe {
        // 执行汇编指令，返回到用户态
        asm!(
            "fence.i",
            "jr {restore_va}",         // 跳转到 __restore_trap 汇编函数的新地址
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,      // a0 = 陷阱上下文的虚拟地址
            in("a1") user_satp,        // a1 = 用户页表的物理地址
            options(noreturn)
        );
    }
}

// 定义一个不进行名称修饰的函数，用于处理来自内核的陷阱
#[unsafe(no_mangle)]
/// 处理来自内核的陷阱
pub fn trap_from_kernel() -> ! {
    // 发生来自内核的陷阱时，触发 panic
    panic!("A trap from kernel occurs!");
}
