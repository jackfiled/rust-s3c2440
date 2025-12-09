use crate::system::{StatusRegister, read_cpsr};
use crate::{MANAGER, debug};
use core::arch::naked_asm;
use core::ops::{Deref, DerefMut};
use rust_s3c2440_hal::interrupt::{InterruptController, InterruptSource};
use rust_s3c2440_hal::utils::Register;

const INTERRUPT_VECTOR_BASE_ADDRESS: usize = 0x3000_0100;

#[repr(C)]
struct InterruptVector {
    undefined_exception_handler: Register,
    software_interrupt_handler: Register,
    prefetch_abort_handler: Register,
    data_abort_handler: Register,
    _reserved: Register,
    interrupt_handler: Register,
    fast_interrupt_handler: Register,
}

pub struct InterruptManager {
    vector: *const InterruptVector,
    controller: InterruptController,
    interrupt_handlers: [fn() -> (); InterruptSource::INTERRUPT_SOURCE_COUNT],
}

impl Deref for InterruptManager {
    type Target = InterruptController;

    fn deref(&self) -> &Self::Target {
        &self.controller
    }
}

impl DerefMut for InterruptManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.controller
    }
}

#[unsafe(naked)]
extern "C" fn set_interrupt_stack() {
    naked_asm!(
        // Enter IRQ mode.
        "mrs r0, cpsr",
        "bic r0, r0, {MODE_MASK}",
        "orr r0, r0, {IRQ_MODE}",
        "msr cpsr, r0",
        // Set the IRQ stack.
        "ldr r0, ={IRQ_STACK}",
        "add sp, r0, {STACK_SIZE}",
        // Back to SCV mode.
        "mrs r0, cpsr",
        "bic r0, r0, {MODE_MASK}",
        "orr r0, r0, {SVC_MODE}",
        "msr cpsr, r0",
        "bx lr",
        MODE_MASK = const rust_s3c2440_hal::s3c2440::CpuMode::MASK,
        IRQ_MODE = const rust_s3c2440_hal::s3c2440::CpuMode::Interrupt as u32,
        IRQ_STACK = sym crate::system::stack::TRAP_STACK,
        STACK_SIZE = const crate::system::stack::STACK_SIZE,
        SVC_MODE = const rust_s3c2440_hal::s3c2440::CpuMode::Management as u32
    );
}

impl InterruptManager {
    pub fn new() -> Self {
        Self {
            vector: INTERRUPT_VECTOR_BASE_ADDRESS as *const InterruptVector,
            controller: InterruptController::new(),
            interrupt_handlers: [|| {}; InterruptSource::INTERRUPT_SOURCE_COUNT],
        }
    }

    pub fn enable_interrupt(&mut self) {
        // 1. Set the interrupt vector.
        self.vector()
            .software_interrupt_handler
            .write(software_interrupt_entry as *const () as u32);
        self.vector()
            .interrupt_handler
            .write(interrupt_entry as *const () as u32);

        // 2. Set the stack point for each mode.
        set_interrupt_stack();

        // 3. Turn on the interrupt.
        let mut cpsr = read_cpsr();
        cpsr.enable_interrupt();
        cpsr.write_cpsr();
    }

    pub fn interrupt_handler(&self, source: InterruptSource) -> fn() -> () {
        self.interrupt_handlers[usize::from(source)]
    }

    pub fn register_interrupt_handler(&mut self, source: InterruptSource, handler: fn() -> ()) {
        self.interrupt_handlers[usize::from(source)] = handler;
    }

    fn vector(&self) -> &InterruptVector {
        unsafe { &(*self.vector) }
    }
}

/// Trap context.
/// The size of context is (15 + 1) * 4 = 64 bytes.
#[repr(C)]
struct TrapContext {
    /// Saved registers, R0 ~ R14.
    /// R13 is the stack pointer, and there is a banked register R13_irq, so the saved value is useless.
    /// R14 is the link register, and there is a banked register R14_irq, the saved value is important.
    /// R15 is PC.
    registers: [u32; 15],
    /// Saved SPSR register.
    status_register: StatusRegister,
}

impl TrapContext {
    fn return_address(&self) -> usize {
        self.registers[14] as usize
    }
}

#[unsafe(naked)]
unsafe extern "C" fn software_interrupt_entry() {
    naked_asm!(
        // 1. Disable interrupt when handling.
        // Before setting, store r0 to use this register.
        "sub sp, sp, #64",
        "stmia sp, {{r0}}",
        "mrs r0, cpsr",
        "orr r0, r0, {I_BIT}",
        "msr cpsr, r0",
        // 2. Store the trap context.
        "add r0, sp, 4",
        "stmia r0!, {{r1- r14}}",
        "mrs r1, spsr",
        "stmia r0!, {{r1}}",
        // 3. Call the rust trap handler.
        // Use the trap context as the argument.
        "mov r0, sp",
        "bl {TRAP_HANDLER}",
        // 4. Restore the environment.
        "mrs r0, spsr",
        "msr cpsr, r0",
        "ldmia sp!, {{r0- r14}}",
        // 5. Call back to the PC.
        // When software interrupt, just go to PC +4.
        "movs pc, r14",
        I_BIT = const crate::system::I_BIT,
        TRAP_HANDLER = sym crate::system::interrupt::software_interrupt_handler
    )
}

fn software_interrupt_handler(context: &TrapContext) {
    debug!("Software interrupt:");
    debug!(
        "\tThe source mode is {}.",
        context.status_register.cpu_mode()
    );
    debug!("\tThe returning PC is {:X}.", context.return_address());
}

#[unsafe(naked)]
unsafe extern "C" fn interrupt_entry() {
    naked_asm!(
        // 1. Disable interrupt when handling.
        // Before setting, store r0 to use this register.
        "sub sp, sp, #64",
        "stmia sp, {{r0}}",
        "mrs r0, cpsr",
        "orr r0, r0, {I_BIT}",
        "msr cpsr, r0",
        // 2. Store the trap context.
        "add r0, sp, 4",
        "stmia r0!, {{r1- r14}}",
        "mrs r1, spsr",
        "stmia r0!, {{r1}}",
        // 3. Call the rust trap handler.
        // Use the trap context as the argument.
        "mov r0, sp",
        "bl {TRAP_HANDLER}",
        // 4. Restore the environment.
        "mrs r0, spsr",
        "msr cpsr, r0",
        "ldmia sp!, {{r0- r14}}",
        // 5. Call back to the PC.
        // For interrupt, go to PC.
        "subs pc, lr, #4",
        I_BIT = const crate::system::I_BIT,
        TRAP_HANDLER = sym crate::system::interrupt::interrupt_handler
    )
}

fn interrupt_handler(context: &TrapContext) {
    debug!(
        "Handle interrupt coming from {} mode and returning address is {}",
        context.status_register.cpu_mode(),
        context.return_address()
    );

    let manager = MANAGER.get().unwrap().interrupt();
    let source = manager.borrow().read_handling();
    debug!("Encounter hardware interrupt: {}", source);

    let handler = manager.borrow().interrupt_handlers[usize::from(source)];
    handler();

    // Clear the pending interrupt.
    manager.borrow_mut().clear_pending_interrupt(source);
    debug!("Hardware interrupt Handled.");
}
