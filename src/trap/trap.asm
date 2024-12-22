.altmacro
.macro STORE_XR regi, offset, base
    sd x\regi, \offset(\base)
.endm
.macro LOAD_XR regi, offset, base
    ld x\regi, \offset(\base)
.endm

.section .text.trampoline
.align 2
.global __save_trap
__save_trap:
    # 交换 sp 和 sscratch
    # 当前 sp 指向用户 *TrapContext，sscratch 指向用户栈
    csrrw sp, sscratch, sp
    # # 为陷阱帧分配空间
    # addi sp, sp, -34*8
    # 保存寄存器
    sd x1, 1*8(sp)
    sd x3, 3*8(sp)

    .set rept_i, 5
    .rept 31-5+1
        STORE_XR %rept_i, %rept_i*8, sp
        .set rept_i, rept_i+1
    .endr

    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    csrr t2, sscratch
    sd t2, 2*8(sp)

    # pub kernel_satp: usize,
    ld t0, 34*8(sp)
    # pub kernel_virt_trap_handler: usize,
    ld t1, 36*8(sp)
    # pub kernel_virt_sp: usize,
    ld sp, 35*8(sp)

    # 切换到内核空间
    csrw satp, t0
    sfence.vma

    # # 调用陷阱处理程序
    # mv a0, sp
    # call trap_handler
    jr t1


.global __restore_trap
__restore_trap: # a0: *TrapContext, a1: 用户空间令牌
    # # 切换到用户空间
    # csrw satp, a1
    # sfence.vma
    # csrw sscratch, a0
    # mv sp, a0
    # # sp 指向 TrapContext
    # # 恢复寄存器
    # ld t0, 32*8(sp)
    # ld t1, 33*8(sp)
    # csrw sstatus, t0
    # csrw sepc, t1
    
    # ld x1, 1*8(sp)
    # ld x3, 3*8(sp)
    # .set rept_i, 5
    # .rept 31-5+1
    #     LOAD_XR %rept_i, %rept_i*8, sp
    #     .set rept_i, rept_i+1
    # .endr

    # # # 释放陷阱帧空间
    # # addi sp, sp, 34*8
    # # # 交换 sp 和 sscratch
    # # csrrw sp, sscratch, sp
    # ld sp, 2*8(sp)
    # sret

    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    # 现在 sp 指向用户空间中的 TrapContext，开始基于它恢复
    # 恢复 sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # 恢复通用寄存器，除了 x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set rept_i, 5
    .rept 31-5+1
        LOAD_XR %rept_i, %rept_i*8, sp
        .set rept_i, rept_i+1
    .endr
    # 返回用户栈
    ld sp, 2*8(sp)

    # 调试测试：从 0x10000 加载 64 位
    lui t0, 0x10
    ld t1, 0(t0)

    sret
