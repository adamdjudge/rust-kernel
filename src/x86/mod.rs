use core::mem;

pub mod gdt;
pub mod io;

/// CPU privilege level. Ring 0 is used by the kernel and ring 3 is used by userspace.
#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PrivilegeLevel {
    Ring0,
    Ring1,
    Ring2,
    Ring3,
}

/// Wrapper for protected mode segment selectors, which consist of an offset into the Global
/// Descriptor Table and a privilege level, and are loaded into the segment registers.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct SegmentSelector(u16);

#[allow(unused)]
impl SegmentSelector {
    /// Creates a new segment selector from a GDT offset and privilege level.
    pub const fn new(offset: u16, dpl: PrivilegeLevel) -> Self {
        Self(offset & !0x7 | dpl as u16)
    }

    /// Returns the segment selector for the GDT's kernel code segment.
    pub const fn kernel_code() -> Self {
        Self::new(
            mem::offset_of!(gdt::Gdt, kernel_code) as u16,
            PrivilegeLevel::Ring0,
        )
    }

    /// Returns the segment selector for the GDT's kernel data segment.
    pub const fn kernel_data() -> Self {
        Self::new(
            mem::offset_of!(gdt::Gdt, kernel_data) as u16,
            PrivilegeLevel::Ring0,
        )
    }

    /// Returns the segment selector for the GDT's user code segment.
    pub const fn user_code() -> Self {
        Self::new(
            mem::offset_of!(gdt::Gdt, user_code) as u16,
            PrivilegeLevel::Ring3,
        )
    }

    /// Returns the segment selector for the GDT's user data segment.
    pub const fn user_data() -> Self {
        Self::new(
            mem::offset_of!(gdt::Gdt, user_data) as u16,
            PrivilegeLevel::Ring3,
        )
    }

    /// Creates a segment selector from a raw u16 value, as read from a segment register.
    pub fn from_u16(value: u16) -> Self {
        Self(value)
    }

    /// Returns the raw u16 value of a segment selector, which can be loaded directly into a segment
    /// register.
    pub fn as_u16(&self) -> u16 {
        self.0
    }

    /// Returns the GDT entry offset portion of the segment selector.
    pub fn offset(&self) -> u16 {
        self.0 & !0x7
    }

    /// Returns the privilege level of the segment selector.
    pub fn dpl(&self) -> PrivilegeLevel {
        unsafe { mem::transmute((self.0 & 0x3) as u8) }
    }

    /// Returns whether the segment belongs to the kernel (privilege level is ring 0).
    pub fn is_kernel(&self) -> bool {
        self.dpl() == PrivilegeLevel::Ring0
    }
}

/// Virtual address wrapper.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VirtAddr(u32);

impl VirtAddr {
    /// Creates a virtual address from a raw u32 value.
    pub const fn new(addr: u32) -> Self {
        Self(addr)
    }

    /// Returns the raw u32 value of a virtual address.
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// Physical address wrapper.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct PhysAddr(u32);

impl PhysAddr {
    /// Creates a new physical address from a raw u32 value.
    pub const fn new(addr: u32) -> Self {
        Self(addr)
    }

    /// Returns the raw u32 value of a physical address.
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}
