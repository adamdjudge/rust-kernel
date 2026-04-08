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

        pub const fn empty() -> Self {
            Self(0)
        }

        pub fn get_carry_flag(&self) -> bool {
            self.0 & Self::CARRY_FLAG.0 != 0
        }

        pub fn set_carry_flag(&mut self, value: bool) -> &mut Self {
            if value {
                self.0 |= Self::CARRY_FLAG.0;
            } else {
                self.0 &= !Self::CARRY_FLAG.0;
            }
            self
        }

        pub fn get_zero_flag(&self) -> bool {
            self.0 & Self::ZERO_FLAG.0 != 0
        }

        pub fn set_zero_flag(&mut self, value: bool) -> &mut Self {
            if value {
                self.0 |= Self::ZERO_FLAG.0;
            } else {
                self.0 &= !Self::ZERO_FLAG.0;
            }
            self
        }

        pub fn get_sign_flag(&self) -> bool {
            self.0 & Self::SIGN_FLAG.0 != 0
        }

        pub fn set_sign_flag(&mut self, value: bool) -> &mut Self {
            if value {
                self.0 |= Self::SIGN_FLAG.0;
            } else {
                self.0 &= !Self::SIGN_FLAG.0;
            }
            self
        }

        pub fn get_trap_flag(&self) -> bool {
            self.0 & Self::TRAP_FLAG.0 != 0
        }

        pub fn set_trap_flag(&mut self, value: bool) -> &mut Self {
            if value {
                self.0 |= Self::TRAP_FLAG.0;
            } else {
                self.0 &= !Self::TRAP_FLAG.0;
            }
            self
        }

        pub fn get_interrupt_flag(&self) -> bool {
            self.0 & Self::INTERRUPT_FLAG.0 != 0
        }

        pub fn set_interrupt_flag(&mut self, value: bool) -> &mut Self {
            if value {
                self.0 |= Self::INTERRUPT_FLAG.0;
            } else {
                self.0 &= !Self::INTERRUPT_FLAG.0;
            }
            self
        }

        pub fn get_overflow_flag(&self) -> bool {
            self.0 & Self::OVERFLOW_FLAG.0 != 0
        }

        pub fn set_overflow_flag(&mut self, value: bool) -> &mut Self {
            if value {
                self.0 |= Self::OVERFLOW_FLAG.0;
            } else {
                self.0 &= !Self::OVERFLOW_FLAG.0;
            }
            self
        }

        pub fn get_virtual_8086_flag(&self) -> bool {
            self.0 & Self::VIRTUAL_8086_FLAG.0 != 0
        }

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

pub mod cr0 {
    use core::arch::asm;
    use core::fmt;

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Cr0(u32);

    #[allow(unused)]
    impl Cr0 {
        pub const PROTECTION_ENABLE_FLAG: Self = Self(1);
        pub const MATH_PRESENT_FLAG: Self = Self(1 << 1);
        pub const EMULATION_FLAG: Self = Self(1 << 2);
        pub const TASK_SWITCHED_FLAG: Self = Self(1 << 3);
        pub const PAGING_FLAG: Self = Self(1 << 31);

        pub fn empty() -> Self {
            Self(0)
        }

        pub fn get_paging_flag(&self) -> bool {
            self.0 & Self::PAGING_FLAG.0 != 0
        }

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

pub mod cr2 {
    use core::arch::asm;

    use crate::x86::VirtAddr;

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

pub mod cr3 {
    use core::arch::asm;

    use crate::x86::VirtAddr;

    pub fn read() -> VirtAddr {
        let value: u32;
        unsafe {
            asm!(
                "mov eax, cr3",
                out("eax") value,
                options(nomem, nostack, preserves_flags)
            );
        }
        VirtAddr::new(value)
    }

    pub unsafe fn write(value: VirtAddr) {
        unsafe {
            asm!(
                "mov cr3, eax",
                in("eax") value.page_align_down().as_u32(),
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}
