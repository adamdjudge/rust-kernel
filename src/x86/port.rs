use core::arch::asm;

macro_rules! port_type {
    ($name:ident, $type:ty, $reg:tt) => {
        /// Representation of an x86 I/O port that can read or write `
        #[doc = stringify!($type)]
        /// ` values.
        #[derive(Debug)]
        pub struct $name(u16);

        #[allow(unused)]
        impl $name {
            /// Returns a new representation of an I/O port.
            #[inline]
            pub const fn new(port: u16) -> Self {
                $name(port)
            }

            /// Reads a value from this I/O port.
            ///
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
    };
}

port_type!(PortU8, u8, "al");
port_type!(PortU16, u16, "ax");
port_type!(PortU32, u32, "eax");
