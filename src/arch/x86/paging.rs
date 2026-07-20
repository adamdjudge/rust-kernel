use core::fmt;
use core::ops::{Index, IndexMut};

use crate::arch::x86::{PhysAddr, VirtAddr};

/// Page size in bytes.
pub const PAGE_SIZE: usize = 4096;
/// Bitmask for aligning an address down to its page base.
pub const PAGE_MASK: u32 = 0xfffff000;

/// An entry in a page table or page directory, which maps a physical frame of memory to a page in
/// a virtual address space. Contains the physical address of the frame and access control flags.
#[repr(transparent)]
pub struct PageTableEntry(u32);

impl PageTableEntry {
    const PRESENT: u32 = 1;
    const WRITABLE: u32 = 1 << 1;
    const USER: u32 = 1 << 2;
    const COPY_ON_WRITE: u32 = 1 << 9;

    /// Creates an empty page table entry, with no address set and all flags cleared.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Creates a new page table entry mapping a physical frame address, with the Present bit set.
    /// Panics if the address is not page-aligned.
    pub fn new(addr: PhysAddr) -> Self {
        assert!(addr.is_page_aligned());
        Self(addr.as_u32() | Self::PRESENT)
    }

    /// Returns the page frame physical address that this page table entry maps to.
    pub fn frame(&self) -> PhysAddr {
        PhysAddr(self.0 & PAGE_MASK)
    }

    /// Returns whether the Present bit is set.
    pub fn is_present(&self) -> bool {
        self.0 & Self::PRESENT != 0
    }

    /// Sets the value of the Present bit. Returns a mutable reference to self for method chaining.
    pub fn set_present(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= Self::PRESENT;
        } else {
            self.0 &= !Self::PRESENT;
        }
        self
    }

    /// Returns the value of the Writable bit.
    pub fn is_writable(&self) -> bool {
        self.0 & Self::WRITABLE != 0
    }

    /// Sets the value of the Writable bit. Returns a mutable reference to self for method chaining.
    pub fn set_writable(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= Self::WRITABLE;
        } else {
            self.0 &= !Self::WRITABLE;
        }
        self
    }

    /// Converts a page table entry to an identical instance with the Writable bit set.
    pub fn with_writable(&self) -> Self {
        Self(self.0 | Self::WRITABLE)
    }

    /// Returns the value of the User bit.
    pub fn is_user(&self) -> bool {
        self.0 & Self::USER != 0
    }

    /// Sets the value of the User bit. Returns a mutable reference to self for method chaining.
    pub fn set_user(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= Self::USER;
        } else {
            self.0 &= !Self::USER;
        }
        self
    }

    /// Converts a page table entry to an identical instance with the User bit set.
    pub fn with_user(&self) -> Self {
        Self(self.0 | Self::USER)
    }

    /// Returns the value of the Copy On Write bit.
    pub fn is_copy_on_write(&self) -> bool {
        self.0 & Self::COPY_ON_WRITE != 0
    }

    /// Sets the value of the Copy on Write bit. Returns a mutable reference to self for method
    /// chaining.
    pub fn set_copy_on_write(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= Self::COPY_ON_WRITE;
        } else {
            self.0 &= !Self::COPY_ON_WRITE;
        }
        self
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PageTableEntry(0x{:08x})", self.0))
    }
}

/// A page table, i.e. a page containing an array of 1024 page table entries. The `PageTable` type
/// may be used as either an L1 page directory or L2 page table.
#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 1024],
}

impl PageTable {
    /// The number of entries in a page table.
    pub const ENTRIES: usize = 1024;

    /// Returns a new empty page table.
    pub const fn new() -> Self {
        Self {
            entries: [const { PageTableEntry::empty() }; Self::ENTRIES],
        }
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

/// A range of pages covered by a virtual address range. This includes all pages containing virtual
/// addresses where `base <= addr < top`.
///
/// When converted to a `PageRangeIterator`, this iterator yields the base virtual address of each
/// page covered by the range.
pub struct PageRange {
    pub base: VirtAddr,
    pub top: VirtAddr,
}

impl PageRange {
    /// Creates a new `PageRange` spanning between two virtual addresses.
    pub const fn new(base: VirtAddr, top: VirtAddr) -> Self {
        Self { base, top }
    }

    /// Creates an iterator over the pages within this range.
    pub fn iter(&self) -> PageRangeIterator {
        PageRangeIterator {
            current: self.base.page_align_down(),
            top: self.top,
        }
    }
}

impl IntoIterator for PageRange {
    type Item = VirtAddr;
    type IntoIter = PageRangeIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct PageRangeIterator {
    current: VirtAddr,
    top: VirtAddr,
}

impl Iterator for PageRangeIterator {
    type Item = VirtAddr;

    fn next(&mut self) -> Option<Self::Item> {
        let page = self.current;
        if page < self.top {
            self.current = self.current.add(PAGE_SIZE);
            Some(page)
        } else {
            None
        }
    }
}
