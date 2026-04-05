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

        pub const fn none() -> Self {
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
                "pushf",
                "pop eax",
                out("eax") value,
                options(preserves_flags)
            );
        }
        Eflags(value)
    }

    pub unsafe fn write(eflags: Eflags) {
        unsafe {
            asm!(
                "push eax",
                "popf",
                in("eax") eflags.0
            );
        }
    }
}
