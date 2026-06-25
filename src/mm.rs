use crate::x86::{
    paging::{PageRange, PageTable, PageTableEntry},
    registers, PhysAddr, VirtAddr,
};

/// Base address of the kernel heap.
pub const HEAP_BASE: VirtAddr = VirtAddr::new(0x04000000);
/// Size of the kernel heap in bytes.
pub const HEAP_SIZE: usize = 64 * 1024 * 1024;

/// Base address of the TCB array.
pub const TCB_BASE: VirtAddr = VirtAddr::new(0x08000000);
/// Size of the TCB array in bytes.
pub const TCB_SIZE: usize = 128 * 1024 * 1024;

/// The dividing line between the userspace and kernelspace portions of the virtual address space.
/// Virtual addresses equal to or above this address belong to userspace, and virtual addresses
/// below belong to the kernel.
pub const USER_BASE: VirtAddr = VirtAddr::new(0x10000000);

pub fn init() {
    // Defined in linker script.
    unsafe extern "C" {
        static __kernel_base: u8;
        static __kernel_rw: u8;
        static __kernel_top: u8;
    }

    let kbase = VirtAddr::from_ptr(&raw const __kernel_base);
    let krw = VirtAddr::from_ptr(&raw const __kernel_rw);
    let ktop = VirtAddr::from_ptr(&raw const __kernel_top);

    /// The page table mapping the kernel executable image within every virtual address space.
    static mut KERNEL_PAGE_TABLE: PageTable = PageTable::new();

    // Identity-map the kernel executable image. The text and rodata segments are mapped read-only,
    // and the data and bss segments are mapped writable.
    for addr in PageRange::new(kbase, krw) {
        unsafe {
            KERNEL_PAGE_TABLE[addr.page_table_index()] = PageTableEntry::new(addr.as_phys());
        }
    }
    for addr in PageRange::new(krw, ktop) {
        unsafe {
            KERNEL_PAGE_TABLE[addr.page_table_index()] =
                PageTableEntry::new(addr.as_phys()).with_writable();
        }
    }

    // Identity-map VGA text buffer.
    unsafe {
        KERNEL_PAGE_TABLE[0xb8] = PageTableEntry::new(PhysAddr::new(0xb8000)).with_writable();
    }

    /// The page table mapping the stack of free physical page frames.
    static mut FRAME_STACK_PAGE_TABLE: PageTable = PageTable::new();

    /// The set of page tables mapping the 64 MiB kernel heap.
    static mut HEAP_PAGE_TABLES: [PageTable; 16] = [const { PageTable::new() }; 16];

    /// The set of page tables mapping the 128 MiB task control block array.
    static mut TCB_PAGE_TABLES: [PageTable; 32] = [const { PageTable::new() }; 32];

    /// The initial page directory used for setting up paging, and then given to the init process.
    ///
    /// Entry 0 is mapped to `KERNEL_PAGE_TABLE`, placing the kernel executable image within
    /// 0x00000000 - 0x003fffff (4 MiB).
    ///
    /// Entry 1 is recursively mapped to the page directory itself, which causes 0x00400000 -
    /// 0x007fffff (4 MiB) to map to an array of all page table entries for the entire virtual
    /// address space.
    ///
    /// Entry 2 is mapped to `FRAME_STACK_PAGE_TABLE`, which maps the stack of free physical page
    /// frames to 0x00800000 - 0x00bfffff (4 MiB).
    /// 
    /// Entries 16 - 31 are mapped to `HEAP_PAGE_TABLES`, which maps the kernel heap to 0x04000000 -
    /// 0x07ffffff (64 MiB).
    ///
    /// Entries 32 - 63 are mapped to `TCB_PAGE_TABLES`, which maps the task control block array to
    /// 0x08000000 - 0x0fffffff (128 MiB).
    static mut INIT_PAGE_DIRECTORY: PageTable = PageTable::new();

    unsafe {
        INIT_PAGE_DIRECTORY[0] =
            PageTableEntry::new(PhysAddr::from_ptr(&raw const KERNEL_PAGE_TABLE)).with_writable();
        INIT_PAGE_DIRECTORY[1] =
            PageTableEntry::new(PhysAddr::from_ptr(&raw const INIT_PAGE_DIRECTORY)).with_writable();
        INIT_PAGE_DIRECTORY[2] =
            PageTableEntry::new(PhysAddr::from_ptr(&raw const FRAME_STACK_PAGE_TABLE))
                .with_writable();

        for i in 0..16 {
            INIT_PAGE_DIRECTORY[16 + i] =
                PageTableEntry::new(PhysAddr::from_ptr(&raw const HEAP_PAGE_TABLES[i]))
                    .with_writable();
        }

        for i in 0..32 {
            INIT_PAGE_DIRECTORY[32 + i] =
                PageTableEntry::new(PhysAddr::from_ptr(&raw const TCB_PAGE_TABLES[i]))
                    .with_writable();
        }
    }

    // Load the page directory into CR3 and enable paging.
    unsafe {
        registers::cr3::write(PhysAddr::from_ptr(&raw const INIT_PAGE_DIRECTORY));

        let mut cr0 = registers::cr0::read();
        cr0.set_paging_flag(true);
        registers::cr0::write(cr0);
    }
}
