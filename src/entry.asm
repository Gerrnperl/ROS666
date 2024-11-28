.section .text.entry
.global _start
_start:
    # Set up the stack pointer
    la sp, bottom_stack_top
    call _kernel_entry

.section .bss.stack
.align 12

.global bottom_stack_bottom
bottom_stack_bottom:
    .space 4096 * 16

.global bottom_stack_top
bottom_stack_top: