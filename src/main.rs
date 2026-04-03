#![no_std]
#![no_main]

mod console;
mod log;
mod paging;
mod sync;
mod x86;

use core::arch::global_asm;
use core::panic::PanicInfo;

use crate::sync::LazyInit;
use crate::x86::gdt::{GdtEntry, GlobalDescriptorTable, TaskStateSegment};
use crate::x86::idt::{exception_handler, InterruptDescriptorTable, InterruptFrame};
use crate::x86::instructions::{cli, hlt, int3};
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

static mut TSS: TaskStateSegment = TaskStateSegment::new();

static GDT: LazyInit<GlobalDescriptorTable> = LazyInit::new(|| GlobalDescriptorTable {
    null_segment: GdtEntry::null(),
    kernel_code: GdtEntry::code_segment(PrivilegeLevel::Ring0),
    kernel_data: GdtEntry::data_segment(PrivilegeLevel::Ring0),
    user_code: GdtEntry::code_segment(PrivilegeLevel::Ring3),
    user_data: GdtEntry::data_segment(PrivilegeLevel::Ring3),
    tss: GdtEntry::tss(unsafe { &*(&raw const TSS) }),
});

static IDT: LazyInit<InterruptDescriptorTable> = LazyInit::new(|| {
    let mut idt = InterruptDescriptorTable::default();
    idt.breakpoint = exception_handler!(breakpoint);
    idt
});

extern "C" fn breakpoint(frame: &InterruptFrame) {
    log_error!("caught breakpoint exception!");
    log_error!("{:?}", frame);
}

#[unsafe(no_mangle)]
fn main() -> ! {
    GDT.load();
    unsafe { &*(&raw const TSS) }.load();
    IDT.load();

    paging::init();
    console::clear();

    log_info!("Hello from Rust kernel!");
    log_info!("Memory in use: {} KB", paging::mem_used() / 1024);

    int3();
    int3();
    int3();

    loop {
        hlt();
    }
}

#[panic_handler]
#[inline(never)]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        log_error!(
            "\nkernel panic at {}:{} - {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        log_error!("\nkernel panic - {}", info.message());
    }

    cli();
    loop {
        hlt();
    }
}
