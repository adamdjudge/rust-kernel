#![no_std]
#![no_main]

mod console;
mod paging;
mod io;

use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;

use crate::console::{Color, Writer};

// This is the entry point to the kernel from the bootloader. It sets the stack pointer and GDT to
// use memory that is within the kernel memory region (plus our GDT needs user and TSS segments),
// and then clears out the BSS and calls main.
// NOTE: Though Intel syntax is the default, we need to use AT&T syntax here to avoid bugs LLVM's
// assembler seems to have regarding the lgdt and jmpl instructions in Intel mode.
global_asm!(
    ".bss
    .align 4

    init_stack:
        .skip 1024
    
    .data
    .align 8

    gdt_start:
        .quad 0x0000000000000000 # Null segment
        .quad 0x00CF9B000000FFFF # Kernel code segment
        .quad 0x00CF93000000FFFF # Kernel data segment
        .quad 0x00CFFB000000FFFF # User code segment
        .quad 0x00CFF3000000FFFF # User data segment
        .quad 0x0000890000000068 # Task state segment
    gdt_end:

    gdt_desc:
        .short gdt_end - gdt_start - 1
        .int gdt_start
    
    .section .text.start
    .global _start
    _start:
        lgdt gdt_desc
        jmpl $0x08, $2f
    2:
        mov $0x10, %ax
        mov %ax, %ds
        mov %ax, %ss
        mov %ax, %es
        mov %ax, %fs
        mov %ax, %gs
        
        mov $__bss_start, %edi
        mov $__bss_end, %ecx
        sub $__bss_start, %ecx
        xor %al, %al
        rep stosb

        mov $init_stack + 1024, %esp
        jmp main",
    options(att_syntax)
);

#[unsafe(no_mangle)]
fn main() -> ! {
    paging::init();

    let writer = Writer::get();
    writer.clear_screen();

    println!("Hello from Rust kernel!");
    println!("Memory in use: {} KB", paging::mem_used() / 1024);

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[panic_handler]
#[inline(never)]
fn panic(info: &PanicInfo) -> ! {
    let mut writer = Writer::get();
    writer.set_bg_color(Color::Black);
    writer.set_text_color(Color::LightRed);

    if let Some(location) = info.location() {
        let _ = write!(
            &mut writer,
            "\nkernel panic at {}:{} - {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        let _ = write!(&mut writer, "\nkernel panic - {}", info.message());
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
