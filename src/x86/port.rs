use core::arch::asm;
use core::fmt;

/// An 8-bit I/O port, which can read or write `u8` values.
pub struct PortU8(u16);

/// A 16-bit I/O port, which can read or write `u16` values.
pub struct PortU16(u16);

/// A 32-bit I/O port, which can read or write `u32` values.
pub struct PortU32(u16);

macro_rules! impl_port {
    ($name:ident, $type:ty, $reg:tt) => {
        #[allow(unused)]
        impl $name {
            /// Returns a new representation of an I/O port.
            #[inline]
            pub const fn new(port: u16) -> Self {
                $name(port)
            }

            /// Reads a value from this I/O port.
            ///
            /// ## Safety
            /// This method is unsafe because accessing peripherals can have side effects that could
            /// violate the memory model.
            #[inline]
            pub unsafe fn read(&self) -> $type {
                let value: $type;
                unsafe {
                    asm!(
                        concat!("in ", $reg, ", dx"),
                        in("dx") self.0, out($reg) value,
                        options(nomem, nostack, preserves_flags)
                    );
                }
                value
            }

            /// Writes a value to this I/O port.
            ///
            /// ## Safety
            /// This method is unsafe because accessing peripherals can have side effects that could
            /// violate the memory model.
            #[inline]
            pub unsafe fn write(&self, value: $type) {
                unsafe {
                    asm!(
                        concat!("out dx, ", $reg),
                        in("dx") self.0, in($reg) value,
                        options(nomem, nostack, preserves_flags)
                    );
                }
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_fmt(format_args!("{}(0x{:04x})", stringify!($name), self.0))
            }
        }
    };
}

impl_port!(PortU8, u8, "al");
impl_port!(PortU16, u16, "ax");
impl_port!(PortU32, u32, "eax");
