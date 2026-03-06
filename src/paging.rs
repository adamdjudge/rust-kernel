use core::arch::asm;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

const PAGE_SIZE: u32 = 4096;
const PAGE_MASK: u32 = 0xfffff000;

const PAGE_PRESENT: u32 = 1 << 0;
const PAGE_WRITABLE: u32 = 1 << 1;
const PAGE_USER: u32 = 1 << 2;
const PAGE_COW: u32 = 1 << 9;

const ENTRIES_PER_TABLE: usize = 1024;

/// Error returned if an attempt to allocate a page fails because the system is out of memory.
#[derive(Debug)]
pub struct OutOfMemoryError;

/// Type of a virtual memory page to map, represented as a set of attribute flags.
#[derive(Debug)]
#[repr(u32)]
pub enum PageType {
    KernelReadonly = PAGE_PRESENT,
    KernelWritable = PAGE_PRESENT | PAGE_WRITABLE,
    UserReadonly = PAGE_PRESENT | PAGE_USER,
    UserWritable = PAGE_PRESENT | PAGE_USER | PAGE_WRITABLE,
}

/// Page table entry. Contains the upper 12 bits of the address of the page, as well as its
/// attribute flags that mark the page present, writable, user-mode accessible, or copy-on-write.
#[derive(Clone, Copy, Default)]
#[repr(transparent)]
pub struct Pte {
    pte: u32,
}

#[allow(dead_code)]
impl Pte {
    /// Returns an empty page table entry.
    pub const fn empty() -> Self {
        Self { pte: 0 }
    }

    /// Returns a new page table entry for the given physical address, with flags set according to
    /// the page type specified.
    pub const fn new(addr: u32, ptype: PageType) -> Self {
        Self {
            pte: addr & PAGE_MASK | ptype as u32,
        }
    }

    /// Returns the base address of the physical page specified by this page table entry.
    pub fn addr(&self) -> u32 {
        self.pte & PAGE_MASK
    }

    /// Returns whether the page is currently marked present.
    pub fn is_present(&self) -> bool {
        self.pte & PAGE_PRESENT != 0
    }

    /// Sets the present attribute of this page to the given setting.
    pub fn set_present(&mut self, setting: bool) {
        if setting {
            self.pte |= PAGE_PRESENT;
        } else {
            self.pte &= !PAGE_PRESENT;
        }
    }

    /// Returns whether the page is currently marked writable.
    pub fn is_writable(&self) -> bool {
        self.pte & PAGE_WRITABLE != 0
    }

    /// Sets the writable attribute of this page to the given setting.
    pub fn set_writable(&mut self, setting: bool) {
        if setting {
            self.pte |= PAGE_WRITABLE;
        } else {
            self.pte &= !PAGE_WRITABLE;
        }
    }

    /// Returns whether the page is currently marked user-mode accessible.
    pub fn is_user(&self) -> bool {
        self.pte & PAGE_USER != 0
    }

    /// Sets the user-mode accessible attribute of this page to the given setting.
    pub fn set_user(&mut self, setting: bool) {
        if setting {
            self.pte |= PAGE_USER;
        } else {
            self.pte &= !PAGE_USER;
        }
    }

    /// Returns whether the page is currently marked copy-on-write.
    pub fn is_cow(&self) -> bool {
        self.pte & PAGE_COW != 0
    }

    /// Sets the copy-on-write attribute of this page to the given setting.
    pub fn set_cow(&mut self, setting: bool) {
        if setting {
            self.pte |= PAGE_COW;
        } else {
            self.pte &= !PAGE_COW;
        }
    }

    /// Returns an optional reference to the page table entry for the given virtual address, or None
    /// if the page table for this address is not mapped into the current page directory.
    pub fn lookup(addr: u32) -> Option<&'static Self> {
        PageTable::page_table_of(addr).map(|pt| pt.get_entry_of(addr))
    }

    /// Returns an optional mutable reference to the page table entry for the given virtual address,
    /// or None if the page table for this address is not mapped into the current page directory.
    pub fn lookup_mut(addr: u32) -> Option<&'static mut Self> {
        PageTable::page_table_of_mut(addr).map(|pt| pt.get_entry_of_mut(addr))
    }
}

#[repr(align(4096))]
struct PageDirectory {
    entries: [Pte; ENTRIES_PER_TABLE],
}

impl PageDirectory {
    /// Returns a reference to the currently mapped page directory.
    fn current() -> &'static Self {
        // SAFETY: This is safe AFTER calling init, as mapping the page directory to itself at index
        // 1 places itself at virtual address 0x401000.
        unsafe { &*(0x401000 as *const Self) }
    }

    /// Returns a mutable reference to the currently mapped page directory.
    fn current_mut() -> &'static mut Self {
        // SAFETY: This is safe AFTER calling init, as mapping the page directory to itself at index
        // 1 places itself at virtual address 0x401000.
        unsafe { &mut *(0x401000 as *mut Self) }
    }

    /// Returns a reference to the page directory entry for the given virtual address.
    fn get_entry_of(&self, addr: u32) -> &Pte {
        &self.entries[directory_index(addr)]
    }

    /// Returns a mutable reference to the page directory entry for the given virtual address.
    fn get_entry_of_mut(&mut self, addr: u32) -> &mut Pte {
        &mut self.entries[directory_index(addr)]
    }
}

#[repr(align(4096))]
struct PageTable {
    entries: [Pte; ENTRIES_PER_TABLE],
}

impl PageTable {
    /// Returns an optional reference to the page table that maps the given virtual address, or None
    /// if no page table is mapped into the current page directory for the address.
    fn page_table_of(addr: u32) -> Option<&'static Self> {
        if PageDirectory::current().get_entry_of(addr).is_present() {
            // SAFETY: This is safe AFTER calling init, as mapping the page directory to itself at
            // entry 1 places an array of all page tables at virtual address 0x400000.
            Some(unsafe { &*(0x400000 as *const Self).add(directory_index(addr)) })
        } else {
            None
        }
    }

    /// Returns an optional mutable reference to the page table that maps the given virtual address,
    /// or None if no page table is mapped into the current page directory for the address.
    fn page_table_of_mut(addr: u32) -> Option<&'static mut Self> {
        if PageDirectory::current().get_entry_of(addr).is_present() {
            // SAFETY: This is safe AFTER calling init, as mapping the page directory to itself at
            // entry 1 places an array of all page tables at virtual address 0x400000.
            Some(unsafe { &mut *(0x400000 as *mut Self).add(directory_index(addr)) })
        } else {
            None
        }
    }

    /// Returns a reference to the page directory entry for the given virtual address.
    fn get_entry_of(&self, addr: u32) -> &Pte {
        &self.entries[table_index(addr)]
    }

    /// Returns a mutable reference to the page directory entry for the given virtual address.
    fn get_entry_of_mut(&mut self, addr: u32) -> &mut Pte {
        &mut self.entries[table_index(addr)]
    }
}

struct PageStack {
    top: AtomicUsize,
}

impl PageStack {
    fn push(&self, addr: u32) {
        let base = 0x800000 as *mut u32;
        // SAFETY: Page stack is mapped at 0x800000 after calling init. init ensures enough pages
        // are allocated to the stack to store all physical page addresses to be pushed.
        unsafe {
            *base.add(self.top.fetch_add(1, Ordering::SeqCst)) = addr;
        }
        PAGES_USED.fetch_sub(1, Ordering::SeqCst);
    }

    fn pop(&self) -> Result<u32, OutOfMemoryError> {
        // TODO: critical section
        let base = 0x800000 as *mut u32;
        match self.top.load(Ordering::SeqCst) {
            0 => Err(OutOfMemoryError),
            i => {
                PAGES_USED.fetch_add(1, Ordering::SeqCst);
                self.top.fetch_sub(1, Ordering::SeqCst);
                Ok(unsafe { *base.add(i - 1) })
            }
        }
    }
}

/// Returns the page directory index for the given virtual address.
fn directory_index(addr: u32) -> usize {
    (addr >> 22) as usize
}

/// Returns the page table index for the given virtual address.
fn table_index(addr: u32) -> usize {
    (addr >> 12) as usize % ENTRIES_PER_TABLE
}

/// Flushes the TLB by reloading CR3.
fn flush_tlb() {
    unsafe {
        asm!("mov eax, cr3", "mov cr3, eax");
    }
}

/// Count of the total number of pages in use.
static PAGES_USED: AtomicUsize = AtomicUsize::new(0);

/// Physical page allocation stack.
static PAGE_STACK: PageStack = PageStack {
    top: AtomicUsize::new(0),
};

/// Initializes the paging subsystem by mapping the kernel into virtual memory and enabling paging.
/// Panics if called more than once.
pub fn init() {
    static CALLED: AtomicBool = AtomicBool::new(false);
    if CALLED.swap(true, Ordering::SeqCst) {
        panic!("paging::init was called twice");
    }

    // Initial page directory used for setting up paging, then given to the idle process.
    static mut INIT_PAGE_DIRECTORY: PageDirectory = PageDirectory {
        entries: [Pte::empty(); ENTRIES_PER_TABLE],
    };

    // Initial page table used for setting up paging. Represents the bottom 4 MiB of memory.
    static mut INIT_PAGE_TABLE: PageTable = PageTable {
        entries: [Pte::empty(); ENTRIES_PER_TABLE],
    };

    // Page table used for allocating the physical page stack.
    static mut PAGE_STACK_PAGE_TABLE: PageTable = PageTable {
        entries: [Pte::empty(); ENTRIES_PER_TABLE],
    };

    unsafe extern "C" {
        static __kernel_base: u8;
        static __kernel_top: u8;
    }

    // Map kernel image into virtual memory.
    let mut addr = &raw const __kernel_base as u32;
    let top = &raw const __kernel_top as u32;
    while addr < top {
        unsafe {
            INIT_PAGE_TABLE.entries[table_index(addr)] = Pte::new(addr, PageType::KernelWritable);
        }
        addr += PAGE_SIZE;
        PAGES_USED.fetch_add(1, Ordering::SeqCst);
    }

    unsafe {
        // Map the initial page table and page stack page table into the initial page directory, as
        // well as the initial page directory itself.
        INIT_PAGE_DIRECTORY.entries[0] =
            Pte::new(&raw const INIT_PAGE_TABLE as u32, PageType::KernelWritable);
        INIT_PAGE_DIRECTORY.entries[1] = Pte::new(
            &raw const INIT_PAGE_DIRECTORY as u32,
            PageType::KernelWritable,
        );
        INIT_PAGE_DIRECTORY.entries[2] = Pte::new(
            &raw const PAGE_STACK_PAGE_TABLE as u32,
            PageType::KernelWritable,
        );

        // Load the page directory into CR3, then enable paging in CR0.
        asm!(
            "mov cr3, eax
             mov eax, cr0
             or eax, 0x80000000
             mov cr0, eax",
            in("eax") &raw const INIT_PAGE_DIRECTORY
        );
    }

    // Just map a page stack large enough for 8 MB for now, using physical pages after kernel image.
    // TODO: Calculate page stack size based on actual memory size.
    for i in 0..2 {
        unsafe {
            PAGE_STACK_PAGE_TABLE.entries[i] = Pte::new(addr, PageType::KernelWritable);
        }
        addr += PAGE_SIZE;
        PAGES_USED.fetch_add(1, Ordering::SeqCst);
    }

    // Push the next 8 MB of physical pages to the page stack.
    // TODO: Push all available pages from actual memory map.
    for _ in 0..2048 {
        PAGE_STACK.push(addr);
        addr += PAGE_SIZE;

        // Cancel out PAGES_USED decrement caused by pushing these pages.
        PAGES_USED.fetch_add(1, Ordering::SeqCst);
    }

    // Map VGA text memory.
    map_page(0xb8000, 0xb8000, PageType::KernelWritable).unwrap();
}

/// Returns the amount of system memory currently in use in bytes, based on the number of physical
/// pages allocated.
pub fn mem_used() -> usize {
    PAGES_USED.load(Ordering::SeqCst) * PAGE_SIZE as usize
}

/// Maps the physical page at address `paddr` to the virtual address `vaddr`, with flags set
/// according to the page type specified. Because a page allocation may be required if there is not
/// already a page table mapped for `vaddr`, this function can return `Err(OutOfMemoryError)`.
pub fn map_page(vaddr: u32, paddr: u32, ptype: PageType) -> Result<(), OutOfMemoryError> {
    let pt_pte = PageDirectory::current_mut().get_entry_of_mut(vaddr);
    if !pt_pte.is_present() {
        *pt_pte = Pte::new(PAGE_STACK.pop()?, PageType::KernelWritable);
    }

    *Pte::lookup_mut(vaddr).unwrap() = Pte::new(paddr, ptype);
    Ok(())
}

/// Allocates a physical page and maps it to the virtual address `vaddr`, with flags set according
/// to the page type specified. Returns `Ok(())` on success, or `Err(OutOfMemoryError)` if there are
/// no more free pages.
pub fn alloc_page(vaddr: u32, ptype: PageType) -> Result<(), OutOfMemoryError> {
    map_page(vaddr, PAGE_STACK.pop()?, ptype)
}

/// Unmaps and frees the physical page mapped to the virtual address `vaddr`. This function has no
/// effect if there is no page currently mapped to `vaddr`.
pub fn free_page(vaddr: u32) {
    if let Some(pte) = Pte::lookup_mut(vaddr) {
        PAGE_STACK.push(pte.addr());
        pte.set_present(false);
    }
}

/// Returns the physical page address mapped to the virtual address `vaddr`, or `None` if `vaddr` is
/// not currently mapped.
pub fn vtophys(vaddr: u32) -> Option<u32> {
    match Pte::lookup(vaddr) {
        Some(pte) if pte.is_present() => Some(pte.addr()),
        _ => None,
    }
}
