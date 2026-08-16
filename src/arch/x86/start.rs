//! The kernel entry point when compiled for x86.
//!
//! This module defines the `_start` symbol, which the bootloader jumps to after loading the kernel
//! executable into memory. `_start` is an assembly language routine which clears the BSS, sets the
//! stack pointer to a temporary stack used during initialization, and finally jumps to the
//! `start_rust` function, also defined here, to begin Rust code execution. `start_rust` then
//! performs platform-specific initialization before calling `kmain` to kick off high-level kernel
//! initialization and start running user processes.
//!
//! The platform-dependent initialization that must be performed here includes:
//! - Setting up the Global Descriptor Table (GDT) and Task State Segment (TSS)
//! - Setting up the Interrupt Descriptor Table (IDT) to point to exception/interrupt entry point
//!   functions defined here, which call higher level kernel functions for platform-independent
//!   exception/interrupt handling
//! - Configuring the interrupt controller and timer interrupts
//! - Initializing the kernel's virtual address space and enabling virtual address translation
//! - Clearing the console and configuring serial output for debug logging

use core::arch::global_asm;

use crate::console;
use crate::kmain;
use crate::sync::LazyInit;
use crate::{log_error, log_info};

use super::chipset;
use super::gdt::{GdtEntry, GlobalDescriptorTable, TaskStateSegment};
use super::idt::{exception_handler, interrupt_handler, page_fault_handler};
use super::idt::{InterruptDescriptorTable, InterruptFrame};
use super::{PrivilegeLevel, VirtAddr};

// The entry point to the kernel from the bootloader. Here we clear out the BSS and setup a stack
// for the rest of the initialization code, and then jump to start_rust.
global_asm!(
    ".bss
    .align 4
    init_stack:
        .skip 8192
    
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

        mov $init_stack + 8192, %esp
        jmp start_rust",
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

#[unsafe(no_mangle)]
fn start_rust() -> ! {
    GDT.load();
    let tss = unsafe { &*(&raw const TSS) };
    tss.load();
    IDT.load();

    chipset::pic::init();
    chipset::pit::set_rate(1000);

    console::clear();
    kmain();
}

extern "C" fn breakpoint(frame: &InterruptFrame) {
    log_error!("caught breakpoint exception!");
    log_error!("{:?}", frame);
}

extern "C" fn page_fault(addr: VirtAddr, frame: &InterruptFrame) {
    panic!(
        "caught page fault! err=0x{:x} addr=0x{:x}",
        frame.err,
        addr.as_u32()
    );
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

    chipset::pic::end_of_interrupt(0);
}
