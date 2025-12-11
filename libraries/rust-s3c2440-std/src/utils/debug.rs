use crate::debug;
use crate::system::read_cpsr;
use core::arch::asm;

pub fn print_debug_info() {
    let sp: u32;
    unsafe {
        asm!(
        "mov {}, sp",
        out(reg) sp
        );
    }
    debug!("Register SP: 0x{sp:x}");

    let lr: u32;
    unsafe {
        asm!(
            "mov {}, lr",
            out(reg) lr
        );
    }
    debug!("Register LR: 0x{lr:x}");

    let cpsr = read_cpsr();
    debug!("{cpsr}");
}
