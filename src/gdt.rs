use core::arch::asm;
use core::mem::size_of;

#[repr(C)]
struct GdtEntry {
    limit_lo: u16,
    base_lo: u16,
    base_mid: u8,
    access: u8,
    limit_hi_flags: u8,
    base_hi: u8,
}

impl GdtEntry {
    const fn missing() -> Self {
        Self {
            limit_lo: 0,
            base_lo: 0,
            base_mid: 0,
            access: 0,
            limit_hi_flags: 0,
            base_hi: 0,
        }
    }

    fn set_kernel_code(&mut self) -> &mut Self {
        self.limit_hi_flags = self.limit_hi_flags & 0x0f | 0xc0;
        self.access = 0x9b;
        self
    }

    fn set_kernel_data(&mut self) -> &mut Self {
        self.limit_hi_flags = self.limit_hi_flags & 0x0f | 0xc0;
        self.access = 0x93;
        self
    }

    fn set_user_code(&mut self) -> &mut Self {
        self.limit_hi_flags = self.limit_hi_flags & 0x0f | 0xc0;
        self.access = 0xfb;
        self
    }

    fn set_user_data(&mut self) -> &mut Self {
        self.limit_hi_flags = self.limit_hi_flags & 0x0f | 0xc0;
        self.access = 0xf3;
        self
    }

    fn set_tss(&mut self) -> &mut Self {
        self.limit_hi_flags &= 0x0f;
        self.access = 0x89;
        self
    }

    fn set_base(&mut self, base: u32) -> &mut Self {
        self.base_lo = base as u16;
        self.base_mid = (base >> 16) as u8;
        self.base_hi = (base >> 24) as u8;
        self
    }

    fn set_limit(&mut self, limit: u32) -> &mut Self {
        self.limit_lo = limit as u16;
        self.limit_hi_flags = self.limit_hi_flags & 0xf0 | (limit >> 16) as u8;
        self
    }
}

pub const KERNEL_CS: u16 = 0x0008;
pub const KERNEL_DS: u16 = 0x0010;
pub const USER_CS: u16 = 0x0018;
pub const USER_DS: u16 = 0x0020;
pub const KERNEL_TS: u16 = 0x0028;

#[repr(C)]
struct Gdt {
    null_segment: GdtEntry,
    kernel_code: GdtEntry,
    kernel_data: GdtEntry,
    user_code: GdtEntry,
    user_data: GdtEntry,
    tss: GdtEntry,
}

#[repr(packed)]
struct GdtDescriptor {
    size: u16,
    base: u32,
}

// We need a static GDT descriptor because the lgdt instruction can only use an absolute address.
#[unsafe(no_mangle)]
static mut GDT_DESC: GdtDescriptor = GdtDescriptor { size: 0, base: 0 };

impl Gdt {
    const fn empty() -> Self {
        Self {
            null_segment: GdtEntry::missing(),
            kernel_code: GdtEntry::missing(),
            kernel_data: GdtEntry::missing(),
            user_code: GdtEntry::missing(),
            user_data: GdtEntry::missing(),
            tss: GdtEntry::missing(),
        }
    }

    fn load(&'static self) {
        unsafe {
            GDT_DESC.size = size_of::<Gdt>() as u16 - 1;
            GDT_DESC.base = &raw const *self as u32;

            asm!(
                "lgdt GDT_DESC
                jmpl ${0}, $2f
                2: mov {1:x}, %ds
                mov {1:x}, %ss
                mov {1:x}, %es
                mov {1:x}, %fs
                mov {1:x}, %gs",
                const KERNEL_CS,
                in(reg) KERNEL_DS,
                options(att_syntax)
            );
        }
    }
}

static mut GDT: Gdt = Gdt::empty();

#[repr(C)]
struct Tss {
    unused0: u32,
    esp0: u32,
    ss0: u16,
    unused: [u8; 98],
}

impl Tss {
    const fn empty() -> Self {
        Self {
            unused0: 0,
            esp0: 0,
            ss0: 0,
            unused: [0; 98],
        }
    }
}

static mut TSS: Tss = Tss::empty();

pub fn init() {
    // SAFETY: We only need to touch the GDT during initialization, when there is no multitasking,
    // so let's not bother wrapping it in a mutex.
    let gdt = unsafe { &mut *(&raw mut GDT) };

    gdt.kernel_code.set_kernel_code().set_limit(0xfffff);
    gdt.kernel_data.set_kernel_data().set_limit(0xfffff);
    gdt.user_code.set_user_code().set_limit(0xfffff);
    gdt.user_data.set_user_data().set_limit(0xfffff);
    gdt.tss
        .set_tss()
        .set_base(&raw const TSS as u32)
        .set_limit(size_of::<Tss>() as u32 - 1);

    gdt.load();
}
