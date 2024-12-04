OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = /*TEMPLATE {BASE_ADDRESS}*/;

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

    __text_end = .;

    __rodata_start = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    __rodata_end = .;

    __data_start = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    __data_end = .;

    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    __bss_end = .;
    __app_end = .;

    /DISCARD/ : {
        *(.eh_frame)
        /*TEMPLATE {DEBUG-DISCARD}*/
    }
}