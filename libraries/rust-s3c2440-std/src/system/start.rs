use core::arch::naked_asm;

#[unsafe(naked)]
#[unsafe(link_section = ".text.entry")]
#[unsafe(export_name = "_start")]
unsafe extern "C" fn start() -> ! {
    naked_asm!(
        // Turn off the watchdog.
        "ldr	r0, ={rWTCON_ADR}		/* r0->WTCON */",
        "ldr	r1, ={rWTCON_INIT_VALUE}	/* r1 = WTCON's initValue */",
        "str	r1, [r0]		/* Turn off the watch-dog */",

        // Setup MMU control register.
        "mov	r1, #{MMU_INIT_VALUE}",
        "mcr p15, 0, r1, c1, c0, 0",

        // Drain write buffer.
        "mov r1, #0",
        // Write into MMU control register.
        "mcr	p15, 0, r1, c7, c10, 4",

        // Flush both I and D caches.
        "mcr	p15, 0, r1, c7, c7, 0",

        // Set process id register to zero, which effectively disables the
        // process id remapping feature.
        "mov r1, #0",
        "mcr p15,0, r1, c13, c0, 0",

        // Disable interrupts in processor and switch to SVC32 mode.
        "mrs r1, cpsr",
        "bic r1, r1, #{MASK_MODE}",
        "orr r1, r1, #{MODE_SVC} | {I_BIT} | {F_BIT}",
        "msr cpsr, r1",

        // Disable individual interrupts in the interrupt controller.
        "ldr r1, =0xffffffff",
        "ldr r2, ={INTMASK_ADR}",
        "str r1, [r2]",
        "ldr r2, ={INTSUBMASK_ADR}",
        "str r1, [r2]",

        // Set asynchronous mode for MMU.
        "mrc p15, 0, r2, c1, c0, 0",
        "orr r2, r2, #{MMUCR_ASYNC}",
        "mcr p15, 0, r2, c1, c0, 0",

        // Set PLL lock time.
        "ldr r2, ={LOCKTIME_ADR}",
        "ldr r1, ={LOCKTIME_INITIAL_VALUE}",
        "str r1, [r2]",

        // Set FCLK:HCLK:PCLK = 1:2:4.
        "ldr r2, ={CLOCK_DIV_ADDR}",
        "ldr r1, ={CLOCK_DIV_VALUE}",
        "str r1, [r2]",

        // FCLK is controlled by MPLL value, see MPLL_INITAL_VALUE also.
        "ldr r2, ={MPLL_ADDRESS}",
        "ldr r1, ={MPLL_INITIAL_VALUE}",
        "str r1, [r2]",

        // Set clock control register and slow clock control register.
        "ldr r2, ={CLOCK_ADDRESS}",
        "ldr r1, ={CLOCK_INITIAL_VALUE}",
        "str r1, [r2]",

        "ldr r2, ={SLOW_CLOCK_ADDRESS}",
        "ldr r1, ={SLOW_CLOCK_INITIAL_VALUE}",
        "str r1, [r2]",

        // Set bus width of each bank.
        "ldr r2, ={BUS_WIDTH_ADDRESS}",
        "ldr r1, ={BUS_WIDTH_INITIAL_VALUE}",
        "str r1, [r2]",

        // Setup bank0.
        "ldr r2, ={BANK_ADDRESS0}",
        "ldr r1, =0x00002F50",
        "str r1, [r2]",

        // Setup bank5.
        "ldr r2, ={BANK_ADDRESS5}",
        "ldr r1, =0x0007FFFC",
        "str r1, [r2]",

        // Setup bank6.
        "ldr r2, ={BANK_ADDRESS6}",
        "ldr r1, ={BANK6_INITIAL_VALUE}",
        "str r1, [r2]",

        // Set refresh controller for SDRAM.
        "ldr r2, ={REFRESH_ADDRESS}",
        "ldr r1, ={REFRESH_INITIAL_VALUE}",
        "str r1, [r2]",

        // Set bank size for SDRAM.
        "ldr r2, ={BANKSIZE_ADDRESS}",
        "ldr r1, ={BANKSIZE_INITIAL_VALUE}",
        "str r1, [r2]",

        // Set bank mode.
        "ldr r2, ={MRSRB6_ADDRESS}",
        "ldr r1, ={MRSRB6_INITIAL_VALUE}",
        "str r1, [r2]",

        // Clear the bss segment.
        "ldr r0, =rust_bss_start",
        "ldr r1, =rust_bss_end",
        "mov r2, #0",

        "1:",
        "cmp r0, r1",
        "bge 2f",
        "str r2, [r0], #4",
        "b 1b",

        "2:",
        // Initialize stack pointer, and the stack will grow from high address to low address.
        "ldr r0, ={STACK}",
        "add sp, r0, #{STACK_SIZE}",

        // Call main function in rust.
        "mov fp, #0",
        "mov r0, #{NORMAL_BOOT}",
        // Currently we don't enable thumb mode, so we just jump into main function directly.
        "b {MAIN_FUNC}",

        rWTCON_ADR = const crate::system::WATCHDOG_ADDRESS,
        rWTCON_INIT_VALUE = const crate::system::WATCHDOG_INITIAL_VALUE,
        MMU_INIT_VALUE = const crate::system::MMU_INITIAL_VALUE,
        MASK_MODE = const crate::system::MASK_MODE,
        MODE_SVC = const crate::system::MODE_SVC32,
        I_BIT = const crate::system::I_BIT,
        F_BIT = const crate::system::F_BIT,
        INTMASK_ADR = const crate::system::INTERRUPT_MASK_ADDRESS,
        INTSUBMASK_ADR = const crate::system::INTERRUPT_SUBMASK_ADDRESS,
        MMUCR_ASYNC = const crate::system::MMU_ASYNC,
        LOCKTIME_ADR = const crate::system::LOCKTIME_ADDRESS,
        LOCKTIME_INITIAL_VALUE = const crate::system::LOCKTIME_INITIAL_VALUE,
        CLOCK_DIV_ADDR = const crate::system::CLOCK_DIV_ADDRESS,
        CLOCK_DIV_VALUE = const crate::system::CLOCK_DIV_INITIAL_VALUE,
        MPLL_ADDRESS = const crate::system::MPLL_ADDRESS,
        MPLL_INITIAL_VALUE = const crate::system::MPLL_INITIAL_VALUE,
        CLOCK_ADDRESS = const crate::system::CLOCK_ADDRESS,
        CLOCK_INITIAL_VALUE = const crate::system::CLOCK_INITIAL_VALUE,
        SLOW_CLOCK_ADDRESS = const crate::system::SLOW_CLOCK_ADDRESS,
        SLOW_CLOCK_INITIAL_VALUE = const crate::system::SLOW_CLOCK_INITIAL_VALUE,
        BUS_WIDTH_ADDRESS = const crate::system::BUS_WIDTH_ADDRESS,
        BUS_WIDTH_INITIAL_VALUE = const crate::system::BUS_WIDTH_INITIAL_VALUE,
        BANK_ADDRESS0 = const crate::system::BANK_ADDRESS0,
        BANK_ADDRESS5 = const crate::system::BANK_ADDRESS5,
        BANK_ADDRESS6 = const crate::system::BANK_ADDRESS6,
        BANK6_INITIAL_VALUE = const crate::system::BANK6_INITIAL_VALUE,
        REFRESH_ADDRESS = const crate::system::REFRESH_ADDRESS,
        REFRESH_INITIAL_VALUE = const crate::system::REFRESH_INITIAL_VALUE,
        BANKSIZE_ADDRESS = const crate::system::BANKSIZE_ADDRESS,
        BANKSIZE_INITIAL_VALUE = const crate::system::BANKSIZE_INITIAL_VALUE,
        MRSRB6_ADDRESS = const crate::system::MRSRB6_ADDRESS,
        MRSRB6_INITIAL_VALUE = const crate::system::MRSRB6_INITIAL_VALUE,
        STACK = sym crate::system::stack::ROOT_STACK,
        STACK_SIZE = const crate::system::stack::STACK_SIZE,
        NORMAL_BOOT = const crate::system::BootMode::NORMAL.bits(),
        MAIN_FUNC = sym crate::rust_main
    )
}
