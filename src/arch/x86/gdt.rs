use core::arch::asm;
use core::mem::{offset_of, size_of};

use crate::arch::x86::{PrivilegeLevel, SegmentSelector, VirtAddr};

/// Entry in the Global Descriptor Table specifying a protected mode segment.
#[repr(C)]
#[repr(align(8))]
pub struct GdtEntry {
    limit_lo: u16,
    base_lo: u16,
    base_mid: u8,
    access: u8,
    limit_hi_flags: u8,
    base_hi: u8,
}

impl GdtEntry {
    /// Returns an empty GDT entry that does not represent any valid segment.
    pub const fn null() -> Self {
        Self {
            limit_lo: 0,
            base_lo: 0,
            base_mid: 0,
            access: 0,
            limit_hi_flags: 0,
            base_hi: 0,
        }
    }

    /// Returns a GDT entry for a code segment with the given privilege level. The segment covers
    /// the entire address space.
    pub const fn code_segment(dpl: PrivilegeLevel) -> Self {
        Self {
            limit_lo: 0xffff,
            base_lo: 0,
            base_mid: 0,
            access: 0x9b | (dpl as u8) << 5,
            limit_hi_flags: 0xcf,
            base_hi: 0,
        }
    }

    /// Returns a GDT entry for a data segment with the given privilege level. The segment covers
    /// the entire address space.
    pub const fn data_segment(dpl: PrivilegeLevel) -> Self {
        Self {
            limit_lo: 0xffff,
            base_lo: 0,
            base_mid: 0,
            access: 0x93 | (dpl as u8) << 5,
            limit_hi_flags: 0xcf,
            base_hi: 0,
        }
    }

    /// Returns a GDT entry for a ring 0 TSS segment pointing to the given static TSS structure.
    pub fn tss(tss: &'static TaskStateSegment) -> Self {
        let base = tss as *const TaskStateSegment as u32;
        Self {
            limit_lo: size_of::<TaskStateSegment>() as u16 - 1,
            base_lo: base as u16,
            base_mid: (base >> 16) as u8,
            access: 0x89,
            limit_hi_flags: 0x40,
            base_hi: (base >> 24) as u8,
        }
    }
}

/// Global Descriptor Table, necessary for x86 protected mode. This struct contains entries for
/// kernel and user code and data segments, as well as a TSS segment.
#[repr(C)]
pub struct GlobalDescriptorTable {
    pub null_segment: GdtEntry,
    pub kernel_code: GdtEntry,
    pub kernel_data: GdtEntry,
    pub user_code: GdtEntry,
    pub user_data: GdtEntry,
    pub tss: GdtEntry,
}

#[repr(packed)]
struct GdtPointer {
    #[allow(unused)]
    size: u16,
    base: u32,
}

// We need a static GDT descriptor because the lgdt instruction can only use an absolute address.
#[unsafe(no_mangle)]
static mut GDT_PTR: GdtPointer = GdtPointer {
    size: size_of::<GlobalDescriptorTable>() as u16 - 1,
    base: 0,
};

impl GlobalDescriptorTable {
    /// Uses the `lgdt` instruction to load the global descriptor table, as well as set the segment
    /// registers to select kernel_code and kernel_data.
    pub fn load(&'static self) {
        unsafe {
            GDT_PTR.base = &raw const *self as u32;
            asm!(
                "lgdt GDT_PTR",
                "jmpl ${0}, $2f",
                "2: mov {1:x}, %ds",
                "mov {1:x}, %ss",
                "mov {1:x}, %es",
                "mov {1:x}, %fs",
                "mov {1:x}, %gs",
                const offset_of!(Self, kernel_code),
                in(reg) offset_of!(Self, kernel_data),
                options(att_syntax)
            );
        }
    }
}

/// Task State Segment, necessary for x86 multitasking. Only the SS0 and ESP0 fields are needed, as
/// they define the stack segment and pointer to load on switch from user mode to kernel mode.
#[repr(C)]
pub struct TaskStateSegment {
    unused0: u32,
    /// New stack pointer loaded upon privilege escalation to ring 0.
    pub esp0: VirtAddr,
    /// Stack segment loaded upon privilege escalation to ring 0.
    pub ss0: SegmentSelector,
    unused: [u8; 98],
}

impl TaskStateSegment {
    /// Returns a new task state segment. ss0 is initialized to the kernel data segment, but esp0 is
    /// not set.
    pub const fn new() -> Self {
        Self {
            unused0: 0,
            esp0: VirtAddr::null(),
            ss0: SegmentSelector::kernel_data(),
            unused: [0; 98],
        }
    }

    /// Uses the ltr instruction to load the task state segment.
    pub fn load(&'static self) {
        crate::arch::x86::instructions::ltr(SegmentSelector::tss());
    }
}
