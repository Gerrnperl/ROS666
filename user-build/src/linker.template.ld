OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x10000;

SECTIONS
{
    /* put entry at the start */
    . = BASE_ADDRESS;
    __app_start = .;
    __text_start = .;
    .text : {
        *(.text.entry)
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

    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
    . = ALIGN(4K);
    __bss_end = .;
    __app_end = .;

    /DISCARD/ : {
        *(.eh_frame)
        /*TEMPLATE {DEBUG-DISCARD}*/
    }
}