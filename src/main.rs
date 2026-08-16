#![no_std]
#![no_main]

mod arch;
mod console;
mod log;
mod mm;
mod sync;

use core::panic::PanicInfo;

use crate::arch::{disable_interrupts, enable_interrupts, halt};

fn kmain() -> ! {
    log_info!("Hello from Rust kernel!");

    enable_interrupts();
    loop {
        halt();
    }
}

#[panic_handler]
#[inline(never)]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        log_error!(
            "kernel panic at {}:{} - {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        log_error!("kernel panic - {}", info.message());
    }

    disable_interrupts();
    loop {
        halt();
    }
}
