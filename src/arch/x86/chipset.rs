//! Contains modules for working with x86-specific chipset components outside the CPU.

/// Control over the Programmable Interrupt Controllers (PICs).
pub mod pic {
    use crate::arch::x86::port::PortU8;

    const PRIMARY_CMD: PortU8 = PortU8::new(0x20);
    const PRIMARY_DATA: PortU8 = PortU8::new(0x21);
    const SECONDARY_CMD: PortU8 = PortU8::new(0xA0);
    const SECONDARY_DATA: PortU8 = PortU8::new(0xA1);

    const EOI: u8 = 0x20;

    /// Initializes the PICs and configure their IRQs to trigger interrupt vectors 32-47.
    pub fn init() {
        unsafe {
            // Start initialization sequence, indicating ICW4 will be present.
            PRIMARY_CMD.write(0x11);
            SECONDARY_CMD.write(0x11);

            // Vector offset for each PIC.
            PRIMARY_DATA.write(0x20);
            SECONDARY_DATA.write(0x28);

            // Configure secondary PIC cascade on IRQ pin 2.
            PRIMARY_DATA.write(0x04);
            SECONDARY_DATA.write(0x02);

            // Configure 8086 mode.
            PRIMARY_DATA.write(0x01);
            SECONDARY_DATA.write(0x01);

            // Clear interrupt masks.
            PRIMARY_DATA.write(0x00);
            SECONDARY_DATA.write(0x00);
        }
    }

    /// Disables both PICs by masking all interrupts.
    #[allow(unused)]
    pub fn disable() {
        unsafe {
            PRIMARY_DATA.write(0xff);
            SECONDARY_DATA.write(0xff);
        }
    }

    /// Sends an End of Interrupt command either to just the primary PIC, or to both PICs, depending
    /// on the IRQ number provided. This function must be called at the end of all interrupt handler
    /// functions in order to re-enable interrupts.
    pub fn end_of_interrupt(irq: u8) {
        if irq > 7 {
            unsafe {
                SECONDARY_CMD.write(EOI);
            }
        }
        unsafe {
            PRIMARY_CMD.write(EOI);
        }
    }
}

/// Control over the Programmable Interval Timer (PIT).
pub mod pit {
    use crate::arch::x86::port::PortU8;

    const DATA: PortU8 = PortU8::new(0x40);
    const CMD: PortU8 = PortU8::new(0x43);

    const OSC_HZ: usize = 1_193_182;

    /// Sets the timer rate in Hz.
    ///
    /// Panics if the rate is 0 or greater than the oscillator frequency of 1,193,182 Hz.
    pub fn set_rate(hz: usize) {
        if hz == 0 || hz > OSC_HZ {
            panic!("set_rate: invalid timer rate");
        }

        let mut divider = OSC_HZ / hz;
        if divider > u16::MAX as usize {
            divider = u16::MAX as usize;
        }

        unsafe {
            // Set mode 3 (square wave) with 16-bit count.
            CMD.write(0x36);

            // Write divier high and low bytes.
            DATA.write(divider as u8);
            DATA.write((divider >> 8) as u8);
        }
    }
}
