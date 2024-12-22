//! 陷阱上下文模块。

use core::arch::asm;

/// 表示陷阱上下文的结构体。
pub struct TrapCtx {
    pub x: [usize; 32],                  // 通用寄存器
    pub sstatus: usize,                  // 监管者状态寄存器
    pub sepc: usize,                     // 监管者异常程序计数器
    pub kernel_satp: usize,              // 内核页表基址寄存器
    pub kernel_virt_sp: usize,           // 内核虚拟栈指针
    pub kernel_virt_trap_handler: usize, // 内核虚拟陷阱处理程序地址
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
    /// - `kernel_virt_trap_handler`：内核虚拟陷阱处理程序的地址。
    /// ## 返回值
    /// 返回一个初始化的 TrapCtx 实例。
    pub fn init_app_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_virt_sp: usize,
        kernel_virt_trap_handler: usize,
    ) -> Self {
        // riscv crate 不提供从 sstatus 获取位的方法，所以我们必须使用内联汇编
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

/// 实现 `Riscv64RegAlias` 特性用于 `TrapCtx` 结构体。
///
/// 该特性提供了一组方法来访问和修改 RISC-V 64 位架构的寄存器。
impl Riscv64RegAlias for TrapCtx {
    /// # zero:
    ///   返回对 `x[0]` 寄存器的可变引用。
    fn zero(&mut self) -> &mut usize {
        &mut self.x[0]
    }
    /// # ra:
    ///   返回对 `x[1]` 寄存器的可变引用。
    fn ra(&mut self) -> &mut usize {
        &mut self.x[1]
    }
    /// # sp:
    ///   返回对 `x[2]` 寄存器的可变引用。
    fn sp(&mut self) -> &mut usize {
        &mut self.x[2]
    }
    /// # gp:
    ///   返回对 `x[3]` 寄存器的可变引用。
    fn gp(&mut self) -> &mut usize {
        &mut self.x[3]
    }
    /// # tp:
    ///   返回对 `x[4]` 寄存器的可变引用。
    fn tp(&mut self) -> &mut usize {
        &mut self.x[4]
    }
    /// # t:
    ///   返回对 `x[5 + i]` 寄存器的可变引用。
    fn t(&mut self, i: usize) -> &mut usize {
        &mut self.x[5 + i]
    }
    /// # s:
    ///   返回对 `x[13 + i]` 寄存器的可变引用。
    fn s(&mut self, i: usize) -> &mut usize {
        &mut self.x[13 + i]
    }
    /// # a:
    ///   返回对 `x[10 + i]` 寄存器的可变引用。
    fn a(&mut self, i: usize) -> &mut usize {
        &mut self.x[10 + i]
    }
    /// # sepc:
    ///   返回对 `sepc` 寄存器的可变引用。
    fn sepc(&mut self) -> &mut usize {
        &mut self.sepc
    }
    /// # sstatus:
    ///   返回对 `sstatus` 寄存器的可变引用。
    fn sstatus(&mut self) -> &mut usize {
        &mut self.sstatus
    }
}
