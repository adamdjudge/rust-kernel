#![no_std]
#![no_main]

mod console;
mod paging;
mod sync;
mod x86;

use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;

use crate::console::{Color, Writer};
use crate::sync::Initializer;
use crate::x86::gdt::{GdtEntry, GlobalDescriptorTable, TaskStateSegment};
use crate::x86::PrivilegeLevel;

// The entry point to the kernel from the bootloader. Here we clear out the BSS and setup a stack
// for the rest of the initialization code, and then jump to the Rust main function.
global_asm!(
    ".bss
    .align 4
    init_stack:
        .skip 1024
    
    .section .text.start
    .global _start
    _start:
        mov $__bss_start, %edi
        mov $__bss_end, %ecx
        sub $__bss_start, %ecx
        add $3, %ecx
        shr $2, %ecx
        xor %eax, %eax
        rep stosl

        mov $init_stack + 1024, %esp
        jmp main",
    options(att_syntax)
);

static GDT: Initializer<GlobalDescriptorTable> = Initializer::new();

static mut TSS: TaskStateSegment = TaskStateSegment::new();

fn init_segments() {
    let tss = unsafe { &*(&raw const TSS) };

    GDT.initialize(|| GlobalDescriptorTable {
        null_segment: GdtEntry::null(),
        kernel_code: GdtEntry::code_segment(PrivilegeLevel::Ring0),
        kernel_data: GdtEntry::data_segment(PrivilegeLevel::Ring0),
        user_code: GdtEntry::code_segment(PrivilegeLevel::Ring3),
        user_data: GdtEntry::data_segment(PrivilegeLevel::Ring3),
        tss: GdtEntry::tss(tss),
    });

    GDT.get_ref().load();
    tss.load();
}

#[unsafe(no_mangle)]
fn main() -> ! {
    init_segments();
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
