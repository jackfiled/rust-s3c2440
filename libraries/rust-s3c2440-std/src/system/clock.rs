use crate::system::FCLK;
use core::arch::asm;
use rust_s3c2440_hal::nop;

fn delay_cycles(mut cycles: u32) {
    // Group 8 nops once to decrease the cost of loop.
    while cycles > 8 {
        unsafe {
            asm!(
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                "nop",
                options(nomem, nostack, preserves_flags)
            );
        }

        cycles -= 8;
    }

    while cycles > 8 {
        nop();
        cycles -= 1;
    }
}

/// Delay the CPU in micro second unit.
/// The delay is implemented by no-op instruction.
pub fn delay_ms(ms: u32) {
    const MICRO_SECOND: u32 = 10 ^ 6;

    if ms == 0 {
        return;
    }

    // The SoC clock is always higher than 1MHz and one nop instruction will cost one clock,
    // so 1 ms = PCLK / MICRO_SECOND.
    delay_cycles(FCLK / MICRO_SECOND * ms);
}
