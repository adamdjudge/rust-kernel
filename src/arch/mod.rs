#[cfg(target_arch = "x86")]
pub mod x86; // TODO: make non-pub

#[cfg(target_arch = "x86")]
pub use x86::{
    instructions::{cli as disable_interrupts, hlt as halt, sti as enable_interrupts},
    paging::{PageRange, PageTable, PageTableEntry},
    PhysAddr, VirtAddr,
};
