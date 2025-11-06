pub(crate) const STACK_SIZE: usize = 0x4000;

/// Stack type used for trap handler.
#[repr(C, align(64))]
pub(crate) struct Stack([u8; STACK_SIZE]);

impl Stack {
    const ZERO: Self = Self([0; STACK_SIZE]);
}

#[unsafe(link_section = ".bss.stack")]
pub(crate) static mut ROOT_STACK: Stack = Stack::ZERO;
