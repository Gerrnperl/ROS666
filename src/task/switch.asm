# 任务切换

.altmacro
.macro STORE regi, offset, base
    sd s\regi, \offset(\base)
.endm
.macro LOAD regi, offset, base
    ld s\regi, \offset(\base)
.endm

.section .text
.global __switch
__switch: # a0: current_task_ctx_ptr, a1: next_task_ctx_ptr
    # 在当前任务上下文空间里保存 CPU 当前的寄存器快照
    sd ra, 0*8(a0)
    sd sp, 1*8(a0)
    .set rept_i, 0
    .rept 12 # s0~s11
        .set plus2, 2 + rept_i
        STORE %rept_i, %plus2*8, a0
        .set rept_i, rept_i+1
    .endr

    # 恢复下一个任务的寄存器
    ld ra, 0*8(a1)
    ld sp, 1*8(a1)
    .set rept_i, 0
    .rept 12 # s0~s11
        .set plus2, 2 + rept_i
        LOAD %rept_i, %plus2*8, a1
        .set rept_i, rept_i+1
    .endr

    ret
