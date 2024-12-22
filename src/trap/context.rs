//! 陷入上下文模块。

use core::arch::asm;

/// 陷入上下文
///
/// 用于保存和恢复陷入处理程序的上下文。
pub struct TrapCtx {
    /// 通用寄存器
    pub x: [usize; 32],
    /// Supervisor 状态寄存器
    pub sstatus: usize,
    /// Supervisor 异常程序计数器                 
    pub sepc: usize,
    /// 内核页表基址 (Supervisor Address Translation and Protection)
    pub kernel_satp: usize,
    /// 内核虚拟栈指针
    pub kernel_virt_sp: usize,
    /// 内核虚拟陷入处理程序入口地址
    pub kernel_virt_trap_handler: usize,
}

impl TrapCtx {
    /// 设置栈指针寄存器的值。
    pub fn set_sp(&mut self, sp: usize) {
        *self.sp() = sp;
    }

    /// 创建一个新的 TrapCtx，用于应用程序上下文。
    ///
    /// ## 参数
    /// - `entry`：程序入口地址。
    /// - `sp`：栈指针地址。
    /// - `kernel_satp`：内核页表基址寄存器的值。
    /// - `kernel_virt_sp`：内核虚拟栈指针的值。
    /// - `kernel_virt_trap_handler`：内核虚拟陷入处理程序的地址。
    /// ## 返回值
    /// 返回一个初始化的 TrapCtx 实例。
    pub fn init_app_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_virt_sp: usize,
        kernel_virt_trap_handler: usize,
    ) -> Self {
        // the riscv crate does not provide a way to get bits from sstatus
        // so we have to use inline assembly
        let mut sstatus: usize;
        unsafe {
            asm!("csrr {}, sstatus", out(reg) sstatus);
        }
        // 清除 FS 位
        sstatus &= !(1 << 8);
        let mut ctx = Self {
            x: [0; 32],
            sstatus: sstatus,
            sepc: entry,
            kernel_satp,
            kernel_virt_sp,
            kernel_virt_trap_handler,
        };
        ctx.set_sp(sp);
        ctx
    }
}

/// 提供 RISC-V 64 位寄存器的别名。
///
/// 该 trait 定义了访问和修改 RISC-V 64 位架构中各种寄存器的方法。
/// 每个方法返回一个 `usize` 的可变引用，表示寄存器的值。
#[allow(unused)]
pub trait Riscv64RegAlias {
    /// 返回零寄存器（x0）的可变引用，其值始终为 0。
    fn zero(&mut self) -> &mut usize;

    /// 返回返回地址寄存器（ra）的可变引用。
    fn ra(&mut self) -> &mut usize;

    /// 返回栈指针寄存器（x2）的可变引用。
    fn sp(&mut self) -> &mut usize;

    /// 返回全局指针寄存器（gp）的可变引用。
    fn gp(&mut self) -> &mut usize;

    /// 返回线程指针寄存器（tp）的可变引用。
    fn tp(&mut self) -> &mut usize;

    /// 返回一个临时寄存器（t0-t6）的可变引用。
    ///
    /// ## 参数
    /// - `i`：临时寄存器的索引（0-6）。
    fn t(&mut self, i: usize) -> &mut usize;

    /// 返回一个保存寄存器（s0-s11）的可变引用。
    ///
    /// ## 参数
    /// - `i`：保存寄存器的索引（0-11）。
    fn s(&mut self, i: usize) -> &mut usize;

    /// 返回一个参数寄存器（a0-a7）的可变引用。
    ///
    /// ## 参数
    /// - `i`：参数寄存器的索引（0-7）。
    fn a(&mut self, i: usize) -> &mut usize;

    /// 返回一个可变引用，指向监管者异常程序计数器（sepc）。
    fn sepc(&mut self) -> &mut usize;

    /// 返回一个可变引用，指向监管者状态寄存器（sstatus）。
    fn sstatus(&mut self) -> &mut usize;
}

impl Riscv64RegAlias for TrapCtx {
    fn zero(&mut self) -> &mut usize {
        &mut self.x[0]
    }
    fn ra(&mut self) -> &mut usize {
        &mut self.x[1]
    }
    fn sp(&mut self) -> &mut usize {
        &mut self.x[2]
    }
    fn gp(&mut self) -> &mut usize {
        &mut self.x[3]
    }
    fn tp(&mut self) -> &mut usize {
        &mut self.x[4]
    }
    fn t(&mut self, i: usize) -> &mut usize {
        &mut self.x[5 + i]
    }
    fn s(&mut self, i: usize) -> &mut usize {
        &mut self.x[13 + i]
    }
    fn a(&mut self, i: usize) -> &mut usize {
        &mut self.x[10 + i]
    }
    fn sepc(&mut self) -> &mut usize {
        &mut self.sepc
    }
    fn sstatus(&mut self) -> &mut usize {
        &mut self.sstatus
    }
}
