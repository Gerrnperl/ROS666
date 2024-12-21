# 任务切换

.altmacro
.macro STORE_SR regi, offset, base
    sd s\regi, \offset(\base)  # 将寄存器 s\regi 的值存储到基址 \base 的偏移 \offset 处
.endm
.macro LOAD_SR regi, offset, base
    ld s\regi, \offset(\base)  # 从基址 \base 的偏移 \offset 处加载值到寄存器 s\regi
.endm

.section .text
.global __switch
__switch: # a0: current_task_ctx_ptr, a1: next_task_ctx_ptr
    # 在当前任务上下文空间里保存 CPU 当前的寄存器快照
    sd ra, 0*8(a0)  # 保存返回地址寄存器 ra
    sd sp, 1*8(a0)  # 保存堆栈指针寄存器 sp
    .set rept_i, 0
    .rept 12 # s0~s11
        .set plus2, 2 + rept_i
        STORE_SR %rept_i, %plus2*8, a0  # 保存 s0~s11 寄存器
        .set rept_i, rept_i+1
    .endr

    # 恢复下一个任务的寄存器
    ld ra, 0*8(a1)  # 恢复返回地址寄存器 ra
    ld sp, 1*8(a1)  # 恢复堆栈指针寄存器 sp
    .set rept_i, 0
    .rept 12 # s0~s11
        .set plus2, 2 + rept_i
        LOAD_SR %rept_i, %plus2*8, a1  # 恢复 s0~s11 寄存器
        .set rept_i, rept_i+1
    .endr

    ret  # 返回
