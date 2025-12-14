use crate::system::read_cpsr;

pub mod cell;
pub mod debug;

/// Create a critical section with interrupt disabled.
/// The guard will disable interrupt before itself is dropped and when dropping, it will restore
/// the interrupt configuration.
pub struct InterruptGuard {
    interrupt_enabled: bool,
    fast_interrupt_enabled: bool,
}

impl InterruptGuard {
    pub fn new() -> Self {
        let mut cpsr = read_cpsr();

        let (interrupt_enabled, fast_interrupt_enabled) =
            (cpsr.interrupt_enabled(), cpsr.fast_interrupt_enabled());

        cpsr.disable_interrupt();
        cpsr.disable_fast_interrupt();
        cpsr.write_cpsr();

        Self {
            interrupt_enabled,
            fast_interrupt_enabled,
        }
    }
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        let mut cpsr = read_cpsr();

        if self.interrupt_enabled {
            cpsr.enable_interrupt();
        }

        if self.fast_interrupt_enabled {
            cpsr.enable_fast_interrupt();
        }

        cpsr.write_cpsr();
    }
}
