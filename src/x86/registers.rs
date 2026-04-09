//! Contains modules for working with special CPU registers, namely `EFLAGS` and the various control
//! registers.

/// Access to the processor `EFLAGS` register, which contains arithmetic condition flags and system
/// control flags.
pub mod eflags {
    use core::arch::asm;
    use core::fmt;

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Eflags(u32);

    #[allow(unused)]
    impl Eflags {
        pub const CARRY_FLAG: Self = Self(1);
        pub const PARITY_FLAG: Self = Self(1 << 2);
        pub const AUXILIARY_FLAG: Self = Self(1 << 4);
        pub const ZERO_FLAG: Self = Self(1 << 6);
        pub const SIGN_FLAG: Self = Self(1 << 7);
        pub const TRAP_FLAG: Self = Self(1 << 8);
        pub const INTERRUPT_FLAG: Self = Self(1 << 9);
        pub const DIRECTION_FLAG: Self = Self(1 << 10);
        pub const OVERFLOW_FLAG: Self = Self(1 << 11);
        pub const NESTED_FLAG: Self = Self(1 << 14);
        pub const RESUME_FLAG: Self = Self(1 << 16);
        pub const VIRTUAL_8086_FLAG: Self = Self(1 << 17);

        /// Creates a new `Eflags` value with no flags set.
        pub const fn empty() -> Self {
            Self(0)
        }

        /// Returns the raw value of the flag bits.
        pub const fn as_u32(&self) -> u32 {
            self.0
        }

        /// Returns the `CF` (Carry Flag) bit.
        pub fn get_carry_flag(&self) -> bool {
            self.0 & Self::CARRY_FLAG.0 != 0
        }

        /// Sets the state of the `CF` (Carry Flag) bit.
        pub fn set_carry_flag(&mut self, value: bool) -> &mut Self {
            if value {
                self.0 |= Self::CARRY_FLAG.0;
            } else {
                self.0 &= !Self::CARRY_FLAG.0;
            }
            self
        }

        /// Returns the `ZF` (Zero Flag) bit.
        pub fn get_zero_flag(&self) -> bool {
            self.0 & Self::ZERO_FLAG.0 != 0
        }

        /// Sets the state of the `ZF` (Zero Flag) bit.
        pub fn set_zero_flag(&mut self, value: bool) -> &mut Self {
            if value {
                self.0 |= Self::ZERO_FLAG.0;
            } else {
                self.0 &= !Self::ZERO_FLAG.0;
            }
            self
        }

        /// Returns the `SF` (Sign Flag) bit.
        pub fn get_sign_flag(&self) -> bool {
            self.0 & Self::SIGN_FLAG.0 != 0
        }

        /// Sets the state of the `SF` (Sign Flag) bit.
        pub fn set_sign_flag(&mut self, value: bool) -> &mut Self {
            if value {
                self.0 |= Self::SIGN_FLAG.0;
            } else {
                self.0 &= !Self::SIGN_FLAG.0;
            }
            self
        }

        /// Returns the `TF` (Trap Flag) bit.
        pub fn get_trap_flag(&self) -> bool {
            self.0 & Self::TRAP_FLAG.0 != 0
        }

        /// Sets the state of the `TF` (Trap Flag) bit.
        pub fn set_trap_flag(&mut self, value: bool) -> &mut Self {
            if value {
                self.0 |= Self::TRAP_FLAG.0;
            } else {
                self.0 &= !Self::TRAP_FLAG.0;
            }
            self
        }

        /// Returns the `IF` (Interrupt Flag) bit.
        pub fn get_interrupt_flag(&self) -> bool {
            self.0 & Self::INTERRUPT_FLAG.0 != 0
        }

        /// Sets the state of the `IF` (Interrupt Flag) bit.
        pub fn set_interrupt_flag(&mut self, value: bool) -> &mut Self {
            if value {
                self.0 |= Self::INTERRUPT_FLAG.0;
            } else {
                self.0 &= !Self::INTERRUPT_FLAG.0;
            }
            self
        }

        /// Returns the `OF` (Overflow Flag) bit.
        pub fn get_overflow_flag(&self) -> bool {
            self.0 & Self::OVERFLOW_FLAG.0 != 0
        }

        /// Sets the state of the `OF` (Overflow Flag) bit.
        pub fn set_overflow_flag(&mut self, value: bool) -> &mut Self {
            if value {
                self.0 |= Self::OVERFLOW_FLAG.0;
            } else {
                self.0 &= !Self::OVERFLOW_FLAG.0;
            }
            self
        }

        /// Returns the `VM` (Virtual 8086 Mode) bit.
        pub fn get_virtual_8086_flag(&self) -> bool {
            self.0 & Self::VIRTUAL_8086_FLAG.0 != 0
        }

        /// Sets the state of the `VM` (Virtual 8086 Mode) bit.
        pub fn set_virtual_8086_flag(&mut self, value: bool) -> &mut Self {
            if value {
                self.0 |= Self::VIRTUAL_8086_FLAG.0;
            } else {
                self.0 &= !Self::VIRTUAL_8086_FLAG.0;
            }
            self
        }
    }

    impl fmt::Debug for Eflags {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_fmt(format_args!("Eflags(0x{:08x})", self.0))
        }
    }

    /// Reads the `EFLAGS` register and returns its value.
    pub fn read() -> Eflags {
        let value: u32;
        unsafe {
            asm!(
                "pushfd",
                "pop eax",
                out("eax") value,
                options(preserves_flags)
            );
        }
        Eflags(value)
    }

    /// Writes a new value to the `EFLAGS` register.
    /// 
    /// ## Safety
    /// This function is unsafe because modifying certain flags, such as `DF` or the arithmetic
    /// condition flags, can cause undefined behavior in Rust code. Additionally, setting the `VM`
    /// flag enables the virtual 8086 mode of the processor, which could produce undefined behavior
    /// including memory safety violation if the virtual machine environment is not properly set up
    /// beforehand.
    pub unsafe fn write(value: Eflags) {
        unsafe {
            asm!(
                "push eax",
                "popfd",
                in("eax") value.0
            );
        }
    }
}

/// Access to the processor `CR0` register, which contains system control flags.
pub mod cr0 {
    use core::arch::asm;
    use core::fmt;

    /// Represents a `CR0` register value.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Cr0(u32);

    #[allow(unused)]
    impl Cr0 {
        pub const PROTECTION_ENABLE_FLAG: Self = Self(1);
        pub const MATH_PRESENT_FLAG: Self = Self(1 << 1);
        pub const EMULATION_FLAG: Self = Self(1 << 2);
        pub const TASK_SWITCHED_FLAG: Self = Self(1 << 3);
        pub const PAGING_FLAG: Self = Self(1 << 31);

        /// Creates a new `Cr0` value with no flags set.
        pub const fn empty() -> Self {
            Self(0)
        }

        /// Returns the raw value of the flag bits.
        pub const fn as_u32(&self) -> u32 {
            self.0
        }

        /// Returns the `PG` (Paging) bit.
        pub fn get_paging_flag(&self) -> bool {
            self.0 & Self::PAGING_FLAG.0 != 0
        }

        /// Sets the state of the `PG` (Paging) bit.
        pub fn set_paging_flag(&mut self, value: bool) {
            if value {
                self.0 |= Self::PAGING_FLAG.0;
            } else {
                self.0 &= !Self::PAGING_FLAG.0;
            }
        }
    }

    impl fmt::Debug for Cr0 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_fmt(format_args!("Cr0(0x{:08x})", self.0))
        }
    }

    /// Reads the `CR0` register and returns its value.
    pub fn read() -> Cr0 {
        let value: u32;
        unsafe {
            asm!(
                "mov eax, cr0",
                out("eax") value,
                options(nomem, nostack, preserves_flags)
            );
        }
        Cr0(value)
    }

    /// Writes a value to the `CR0` register.
    /// 
    /// ## Safety
    /// This function is unsafe because disabling paging can allow for violations of the memory
    /// safety model, and modifying other control flags can cause undefined behavior.
    pub unsafe fn write(value: Cr0) {
        unsafe {
            asm!(
                "mov cr0, eax",
                in("eax") value.0,
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}

/// Access to the processor `CR2` register, which contains the virtual address of the faulting
/// memory access after a page fault exception.
#[allow(unused)]
pub mod cr2 {
    use core::arch::asm;

    use crate::x86::VirtAddr;

    /// Reads the `CR2` register and returns its virtual address value.
    pub fn read() -> VirtAddr {
        let value: u32;
        unsafe {
            asm!(
                "mov eax, cr2",
                out("eax") value,
                options(nomem, nostack, preserves_flags)
            );
        }
        VirtAddr::new(value)
    }
}

/// Access to the processor `CR3` register, which contains the physical address of the page
/// directory.
pub mod cr3 {
    use core::arch::asm;

    use crate::x86::PhysAddr;

    /// Reads the `CR3` register and returns its physical address value.
    pub fn read() -> PhysAddr {
        let value: u32;
        unsafe {
            asm!(
                "mov eax, cr3",
                out("eax") value,
                options(nomem, nostack, preserves_flags)
            );
        }
        PhysAddr::new(value)
    }

    /// Writes the physical address of a page directory to the `CR3` register. Panics if the address
    /// is not page-aligned.
    /// 
    /// ## Safety
    /// This function is unsafe because improperly configuring the page directory could violate the
    /// memory safety model.
    pub unsafe fn write(addr: PhysAddr) {
        if !addr.is_page_aligned() {
            panic!("tried to write non-page-aligned address to cr3");
        }

        unsafe {
            asm!(
                "mov cr3, eax",
                in("eax") addr.as_u32(),
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}
