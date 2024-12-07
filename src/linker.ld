OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    __kernel_start = .;

    __text_start = .;
    .text : {
        *(.text.entry)
        . = ALIGN(4K);
        __strampoline = .;
        *(.text.trampoline);
        . = ALIGN(4K);
        *(.text .text.*)
    }

    . = ALIGN(4K);
    __text_end = .;
    __rodata_start = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    __rodata_end = .;
    __data_start = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    __data_end = .;
    __bss_start_stack = .;
    .bss : {
        *(.bss.stack)
        __bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    __bss_end = .;
    __kernel_end = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}