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
    # swap sp and sscratch
    # currently sp -> user *TrapContext, sscratch -> user stack
    csrrw sp, sscratch, sp
    # # allocate space for trap frame
    # addi sp, sp, -34*8
    # save registers
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

    # switch to kernel space
    csrw satp, t0
    sfence.vma

    # # call trap handler
    # mv a0, sp
    # call trap_handler
    jr t1


.global __restore_trap
__restore_trap: # a0: *TrapContext, a1: user space token
    # # switch to user space
    # csrw satp, a1
    # sfence.vma
    # csrw sscratch, a0
    # mv sp, a0
    # # sp -> TrapContext
    # # restore registers
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

    # # # free space for trap frame
    # # addi sp, sp, 34*8
    # # # swap sp and sscratch
    # # csrrw sp, sscratch, sp
    # ld sp, 2*8(sp)
    # sret

    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # restore general purpose registers except x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set rept_i, 5
    .rept 31-5+1
        LOAD_XR %rept_i, %rept_i*8, sp
        .set rept_i, rept_i+1
    .endr
    # back to user stack
    ld sp, 2*8(sp)

    # debug test: load 64bits from 0x10000
    lui t0, 0x10
    ld t1, 0(t0)

    sret
