#![no_std]
#![no_main]

mod arch;
mod console;
mod log;
mod mm;
mod sync;

use core::arch::global_asm;
use core::panic::PanicInfo;

use crate::sync::LazyInit;
use crate::arch::x86::gdt::{GdtEntry, GlobalDescriptorTable, TaskStateSegment};
use crate::arch::x86::idt::{exception_handler, interrupt_handler, page_fault_handler};
use crate::arch::x86::idt::{InterruptDescriptorTable, InterruptFrame};
use crate::arch::x86::instructions::{cli, hlt, sti};
use crate::arch::x86::{PrivilegeLevel, VirtAddr};

// The entry point to the kernel from the bootloader. Here we clear out the BSS and setup a stack
// for the rest of the initialization code, and then jump to the Rust main function.
global_asm!(
    ".bss
    .align 4
    init_stack:
        .skip 4096
    
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

        mov $init_stack + 4096, %esp
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
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint = exception_handler!(breakpoint);
    idt.page_fault = page_fault_handler!(page_fault);
    idt.irq[0] = interrupt_handler!(timer_handler);
    idt
});

extern "C" fn breakpoint(frame: &InterruptFrame) {
    log_error!("caught breakpoint exception!");
    log_error!("{:?}", frame);
}

extern "C" fn page_fault(addr: VirtAddr, frame: &InterruptFrame) {
    panic!("caught page fault! err=0x{:x} addr=0x{:x}", frame.err, addr.as_u32());
}

static mut COUNTER: usize = 0;

extern "C" fn timer_handler(_: &InterruptFrame) {
    let count = unsafe {
        COUNTER = COUNTER + 1;
        if COUNTER == 1000 {
            COUNTER = 0;
        }
        COUNTER
    };

    if count == 0 {
        log_info!("timer interrupt!");
    }

    arch::x86::chipset::pic::end_of_interrupt(0);
}

#[unsafe(no_mangle)]
fn main() -> ! {
    GDT.load();
    unsafe { &*(&raw const TSS) }.load();
    IDT.load();

    mm::init();
    console::clear();

    arch::x86::chipset::pic::init();
    arch::x86::chipset::pit::set_rate(1000);

    log_info!("Hello from Rust kernel!");

    sti();
    loop {
        hlt();
    }
}

#[panic_handler]
#[inline(never)]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        log_error!(
            "kernel panic at {}:{} - {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        log_error!("kernel panic - {}", info.message());
    }

    cli();
    loop {
        hlt();
    }
}
