use crate::system::FCLK;
use rust_s3c2440_hal::delay_cycles;

/// Delay the CPU in micro second unit.
/// The delay is implemented by no-op instruction.
pub fn delay_ms(ms: u32) {
    const MICRO_SECOND: u32 = 10 ^ 6;

    if ms == 0 {
        return;
    }

    // The SoC clock is always higher than 1MHz and one nop instruction will cost one clock,
    // so 1 ms = FCLK / MICRO_SECOND.
    delay_cycles(FCLK / MICRO_SECOND * ms);
}
