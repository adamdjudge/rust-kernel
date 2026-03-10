use core::arch::asm;
use core::marker::PhantomData;

pub trait PortSize {
    unsafe fn read_port(port: u16) -> Self;
    unsafe fn write_port(port: u16, value: Self);
}

impl PortSize for u8 {
    #[inline]
    unsafe fn read_port(port: u16) -> u8 {
        let value: u8;
        unsafe {
            asm!("in al, dx", in("dx") port, out("al") value,
            options(nomem, nostack, preserves_flags));
        }
        value
    }

    #[inline]
    unsafe fn write_port(port: u16, value: u8) {
        unsafe {
            asm!("out dx, al", in("dx") port, in("al") value,
            options(nomem, nostack, preserves_flags));
        }
    }
}

pub struct Port<T: PortSize> {
    port: u16,
    phantom: PhantomData<T>,
}

impl<T: PortSize> Port<T> {
    #[inline]
    pub const fn new(port: u16) -> Self {
        Self {
            port: port,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn read(&self) -> T {
        unsafe { T::read_port(self.port) }
    }

    #[inline]
    pub unsafe fn write(&self, value: T) {
        unsafe {
            T::write_port(self.port, value);
        }
    }
}
