.altmacro
.macro STORE_XR regi, offset, base
    sd x\regi, \offset(\base)
.endm
.macro LOAD_XR regi, offset, base
    ld x\regi, \offset(\base)
.endm

.align 2
.global __save_trap
__save_trap:
    # swap sp and sscratch
    # currently sp -> user stack, sscratch -> kernel stack
    csrrw sp, sscratch, sp
    # allocate space for trap frame
    addi sp, sp, -34*8
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

    # call trap handler
    mv a0, sp
    call trap_handler


.global __restore_trap
__restore_trap:
    # mv sp, a0
    # restore registers
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set rept_i, 5
    .rept 31-5+1
        LOAD_XR %rept_i, %rept_i*8, sp
        .set rept_i, rept_i+1
    .endr

    # free space for trap frame
    addi sp, sp, 34*8
    # swap sp and sscratch
    csrrw sp, sscratch, sp
    sret
