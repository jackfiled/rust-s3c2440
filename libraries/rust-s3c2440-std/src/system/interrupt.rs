use crate::system::stack::STACK_SIZE;
use crate::system::{StatusRegister, read_cpsr, read_spsr};
use crate::{MANAGER, debug};
use alloc::boxed::Box;
use core::arch::{asm, naked_asm};
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
    software_interrupt_handler: Box<dyn Fn() -> ()>,
    interrupt_handlers: [Box<dyn Fn() -> ()>; InterruptSource::INTERRUPT_SOURCE_COUNT],
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
        // Read the cpsr, and the r0 stores original cpsr.
        "mrs r0, cpsr",
        // Clear the mode bits.
        "bic r1, r0, #{MODE_MASK}",
        // Disable interrupt, now the r1 sores a 'clean' cpsr.
        "orr r1, r1, #{I_BIT}",
        // Enter IRQ mode.
        "orr r2, r1, #{IRQ_MODE}",
        "msr cpsr, r2",
        // Set the IRQ stack.
        "ldr sp, ={IRQ_STACK}",
        "add sp, sp, #{STACK_SIZE}",
        // Enter ABT mode.
        "orr r2, r1, #{ABT_MODE}",
        "msr cpsr, r2",
        // Set the ABT using the IRQ stack.
        "ldr sp, ={IRQ_STACK}",
        "add sp, sp, #{STACK_SIZE}",
        // Enter UND mode.
        "orr r2, r1, #{UND_MODE}",
        "msr cpsr, r2",
        // Set the UND using the IRQ stack.
        "ldr sp, ={IRQ_STACK}",
        "add sp, sp, #{STACK_SIZE}",
        // Restore to original mode.
        "msr cpsr, r0",
        "mov pc, lr",
        MODE_MASK = const rust_s3c2440_hal::s3c2440::CpuMode::MASK,
        I_BIT = const crate::system::I_BIT,
        IRQ_MODE = const rust_s3c2440_hal::s3c2440::CpuMode::Interrupt as u32,
        IRQ_STACK = sym crate::system::stack::TRAP_STACK,
        STACK_SIZE = const crate::system::stack::STACK_SIZE,
        ABT_MODE = const rust_s3c2440_hal::s3c2440::CpuMode::Abort as u32,
        UND_MODE = const rust_s3c2440_hal::s3c2440::CpuMode::Undefined as u32
    );
}

impl InterruptManager {
    pub fn new() -> Self {
        Self {
            vector: INTERRUPT_VECTOR_BASE_ADDRESS as *const InterruptVector,
            controller: InterruptController::new(),
            software_interrupt_handler: Self::empty_handler(),
            interrupt_handlers: core::array::from_fn(|_| Self::empty_handler()),
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
        self.vector()
            .undefined_exception_handler
            .write(undefined_exception_entry as *const () as u32);
        self.vector()
            .prefetch_abort_handler
            .write(prefetch_exception_entry as *const () as u32);
        self.vector()
            .data_abort_handler
            .write(data_exception_entry as *const () as u32);
        debug!("Interrupt vector table is set.");

        // 2. Set the stack point for each mode.
        // Before setting the stack, print each stack address.
        let svc_stack: usize;
        unsafe {
            asm!(
                "ldr {}, ={SVC_STACK}",
                out(reg) svc_stack,
                SVC_STACK = sym crate::system::stack::ROOT_STACK
            );
        }
        debug!("The SVC stack is at 0x{:x}", svc_stack + STACK_SIZE);

        let irq_stack: usize;
        unsafe {
            asm!(
            "ldr {}, ={SVC_STACK}",
            out(reg) irq_stack,
            SVC_STACK = sym crate::system::stack::TRAP_STACK
            );
        }
        debug!("The IRQ stack is at 0x{:x}", irq_stack + STACK_SIZE);

        set_interrupt_stack();
        debug!("Interrupt stack has been set.");

        // 3. Turn on the interrupt.
        let mut cpsr = read_cpsr();
        cpsr.enable_interrupt();
        cpsr.write_cpsr();
        debug!("Interrupt has been enabled.");
    }

    pub fn interrupt_handler(&self, source: InterruptSource) -> &Box<dyn Fn() -> ()> {
        &self.interrupt_handlers[usize::from(source)]
    }

    pub fn register_interrupt_handler(
        &mut self,
        source: InterruptSource,
        handler: Box<dyn Fn() -> ()>,
    ) {
        self.controller.enable_interrupt(source);
        self.interrupt_handlers[usize::from(source)] = handler;
    }

    pub fn unregister_interrupt_handler(&mut self, source: InterruptSource) {
        self.controller.disable_interrupt(source);
        self.interrupt_handlers[usize::from(source)] = Self::empty_handler();
    }

    pub fn register_software_interrupt_handler(&mut self, handler: Box<dyn Fn() -> ()>) {
        self.software_interrupt_handler = handler;
    }

    pub fn unregister_software_interrupt_handler(&mut self) {
        self.software_interrupt_handler = Self::empty_handler();
    }

    #[inline]
    fn vector(&self) -> &InterruptVector {
        unsafe { &(*self.vector) }
    }

    fn empty_handler() -> Box<dyn Fn() -> ()> {
        Box::new(|| {})
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

    fn debug_registers(&self) {
        for (i, v) in self.registers.iter().enumerate() {
            debug!("R{}: 0x{:x}", i, v);
        }
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
        // The cpsr register will be restored with spsr.
        "ldmia sp!, {{r0- r14}}",
        // 5. Call back to the PC.
        // When software interrupt, just go to PC +4.
        "movs pc, r14",
        I_BIT = const crate::system::I_BIT,
        TRAP_HANDLER = sym crate::system::interrupt::software_interrupt_handler
    )
}

fn software_interrupt_handler(context: &TrapContext) {
    debug!(
        "Software interrupt, returning PC is 0x{:X}",
        context.return_address()
    );

    let manager = MANAGER.get().unwrap().interrupt();
    let handler = &manager.borrow().software_interrupt_handler;
    handler();

    debug!("Software interrupt handled.");
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
        // The cpsr register will be restored with spsr.
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
        "Handle interrupt with returning address: 0x{:x}",
        context.return_address()
    );
    let cpsr = read_cpsr();
    debug!("Currently {}", cpsr);
    let spsr = read_spsr();
    debug!("Original {}", spsr);

    let manager = MANAGER.get().unwrap().interrupt();
    let source = manager.borrow().read_handling();
    debug!("Encounter hardware interrupt: {}", source);

    let handler = &manager.borrow().interrupt_handlers[usize::from(source)];
    handler();

    // Clear the pending interrupt.
    manager.borrow_mut().clear_pending_interrupt(source);
    debug!("Hardware interrupt Handled.");
}

#[unsafe(naked)]
unsafe extern "C" fn undefined_exception_entry() {
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
        // The cpsr register will be restored with spsr.
        "ldmia sp!, {{r0- r14}}",
        // 5. Call back to the PC.
        // For undefined, go to PC + 4.
        "movs pc, lr",
        I_BIT = const crate::system::I_BIT,
        TRAP_HANDLER = sym crate::system::interrupt::undefined_exception_handler
    )
}

fn undefined_exception_handler(context: &TrapContext) {
    debug!(
        "Handle undefined exception coming from {} mode in {} mode and returning address is 0x{:x}",
        context.status_register.cpu_mode(),
        read_cpsr().cpu_mode(),
        context.return_address()
    );

    panic!("Meet undefined exception!");
}

#[unsafe(naked)]
unsafe extern "C" fn prefetch_exception_entry() {
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
        // The cpsr register will be restored with spsr.
        "ldmia sp!, {{r0- r14}}",
        // 5. Call back to the PC.
        // For prefetch exception, go to PC.
        "subs pc, lr, #4",
        I_BIT = const crate::system::I_BIT,
        TRAP_HANDLER = sym crate::system::interrupt::prefetch_exception_handler
    )
}

fn prefetch_exception_handler(context: &TrapContext) {
    debug!(
        "Handle prefetch instruction exception coming from {} mode in {} mode and returning address is 0x{:x}",
        context.status_register.cpu_mode(),
        read_cpsr().cpu_mode(),
        context.return_address()
    );

    panic!("Meet prefetch instruction exception!");
}

#[unsafe(naked)]
unsafe extern "C" fn data_exception_entry() {
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
        // The cpsr register will be restored with spsr.
        "ldmia sp!, {{r0- r14}}",
        // 5. Call back to the PC.
        // For data exception, go to PC - 4, which is the real data fetch instruction.
        "subs pc, lr, #8",
        I_BIT = const crate::system::I_BIT,
        TRAP_HANDLER = sym crate::system::interrupt::data_exception_handler
    )
}

fn data_exception_handler(context: &TrapContext) {
    debug!(
        "Handle data exception coming from {} mode in {} mode and returning address is 0x{:x}",
        context.status_register.cpu_mode(),
        read_cpsr().cpu_mode(),
        context.return_address()
    );

    panic!("Meet data exception!");
}
