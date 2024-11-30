/* linker for app */
OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80400000;

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
        start_bss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    end_bss = .;
    __app_end = .;

    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}