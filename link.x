ENTRY(_start)

SECTIONS
{
    . = 0x100000;
    __kernel_base = .;

    .text :
    {
        KEEP(*(.text.start))
        *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata .rodata.*)
    }

    .data :
    {
        *(.data .data.*)
    }

    .bss :
    {
        __bss_start = .;
        *(.bss .bss.*)
    }

    __bss_end = .;
    __kernel_top = .;

    /DISCARD/ :
    {
        *(.comment)
    }
}
