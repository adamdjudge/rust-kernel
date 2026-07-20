#[cfg(feature = "x86")]
pub mod x86; // TODO: make non-pub

#[cfg(feature = "x86")]
pub use x86::{
    paging::{PageRange, PageTable, PageTableEntry},
    PhysAddr, VirtAddr,
};
