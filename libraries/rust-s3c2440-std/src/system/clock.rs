use rust_s3c2440_hal::delay_cycles;

/// Delay the CPU in 1 millisecond unit.
/// The delay is implemented by no-op instruction.
pub fn delay_ms(ms: u32) {
    // In experiment, one `nop` instruction costs about 1 / (210 * 10_000)
    // So 1 millisecond should be 2100 cycles.
    const MAGIC_NUMBER: u32 = 2100;

    if ms == 0 {
        return;
    }

    delay_cycles(MAGIC_NUMBER * ms)
}
