use core::arch::asm;

use crate::x86::SegmentSelector;

#[inline]
pub fn hlt() {
    unsafe {
        asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}

#[inline]
pub fn cli() {
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
}

#[inline]
pub fn sti() {
    unsafe {
        asm!("sti", options(nomem, nostack));
    }
}

#[inline]
pub fn ltr(segment: SegmentSelector) {
    unsafe {
        asm!("ltr ax", in("eax") segment.as_u16(), options(nomem, nostack, preserves_flags));
    }
}
