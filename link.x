ENTRY(_start)

PHDRS
{
    text PT_LOAD;
    data PT_LOAD;
}

SECTIONS
{
    . = 0x100000;
    __kernel_base = .;

    .text :
    {
        KEEP(*(.text.start))
        *(.text .text.*)
    } :text

    .rodata :
    {
        *(.rodata .rodata.*)
    } :text

    .data ALIGN(4K) :
    {
        __kernel_rw = .;
        *(.data .data.*)
    } :data

    .bss :
    {
        __bss_start = .;
        *(.bss .bss.*)
    } :data

    __bss_end = .;
    __kernel_top = .;

    /DISCARD/ :
    {
        *(.comment)
        *(.eh_frame*)
        *(.note .note.*)
    }
}
