use core::arch::asm;

use crate::x86::{PrivilegeLevel, SegmentSelector};

#[derive(Default)]
#[repr(C)]
#[repr(align(8))]
pub struct IdtEntry {
    offset_lo: u16,
    segment: SegmentSelector,
    resvd: u8,
    access: u8,
    offset_hi: u16,
}

impl IdtEntry {
    pub fn exception(dpl: PrivilegeLevel, handler: extern "C" fn() -> !) -> Self {
        Self {
            offset_lo: (handler as u32 & 0xffff) as u16,
            segment: SegmentSelector::kernel_code(),
            resvd: 0,
            access: 0x8f | (dpl as u8) << 5,
            offset_hi: (handler as u32 >> 16) as u16,
        }
    }

    pub fn interrupt(handler: extern "C" fn() -> !) -> Self {
        Self {
            offset_lo: (handler as u32 & 0xffff) as u16,
            segment: SegmentSelector::kernel_code(),
            resvd: 0,
            access: 0x8e,
            offset_hi: (handler as u32 >> 16) as u16,
        }
    }
}

#[derive(Default)]
#[repr(C)]
pub struct InterruptDescriptorTable {
    pub divide_error: IdtEntry,
    pub debug_exception: IdtEntry,
    pub non_maskable_interrupt: IdtEntry,
    pub breakpoint: IdtEntry,
    pub overflow: IdtEntry,
    pub bounds_check: IdtEntry,
    pub invalid_opcode: IdtEntry,
    pub coprocessor_not_available: IdtEntry,
    pub double_fault: IdtEntry,
    pub coprocessor_segment_overrun: IdtEntry,
    pub invalid_tss: IdtEntry,
    pub segment_not_present: IdtEntry,
    pub stack_fault: IdtEntry,
    pub general_protection_fault: IdtEntry,
    pub page_fault: IdtEntry,
    reserved: IdtEntry,
    pub math_fault: IdtEntry,
    pub alignment_check: IdtEntry,
    pub machine_check: IdtEntry,
    pub simd_error: IdtEntry,
    pub virtualization_exception: IdtEntry,
    pub control_protection_exception: IdtEntry,
    unused0: [IdtEntry; 10],
    pub irq: [IdtEntry; 16],
    unused1: [IdtEntry; 16],
    unused2: [IdtEntry; 32],
    unused3: [IdtEntry; 32],
    pub syscall: IdtEntry,
}

#[repr(packed)]
struct IdtPointer {
    #[allow(unused)]
    size: u16,
    base: u32,
}

// We need a static IDT descriptor because the lidt instruction can only use an absolute address.
#[unsafe(no_mangle)]
static mut IDT_PTR: IdtPointer = IdtPointer {
    size: size_of::<InterruptDescriptorTable>() as u16 - 1,
    base: 0,
};

impl InterruptDescriptorTable {
    /// Uses the `lidt` instruction to load the interrupt descriptor table.
    pub fn load(&'static self) {
        unsafe {
            IDT_PTR.base = &raw const *self as u32;
            asm!("lidt IDT_PTR", options(att_syntax));
        }
    }
}
