use crate::s3c2440::{GPACON, GPBCON, GPCCON, GPDCON, GPECON, GPFCON, GPGCON, GPHCON, GPJCON};
use crate::utils::Register;

/// Enumeration contains all ports in S3C2440.
#[derive(Debug, Copy, Clone)]
pub enum Port {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    J,
}

/// GPIO controller registers.
/// Almost all port has three control registers, listed as control, data and pull-up register except
/// GPIO A port.
#[repr(C)]
pub struct GPIOPortController {
    pub control_register: Register,
    pub data_register: Register,
    pub pull_up_register: Register,
}

pub const fn gpio_port_controller(port: Port) -> &'static mut GPIOPortController {
    let controller_ptr = match port {
        Port::A => GPACON as *mut GPIOPortController,
        Port::B => GPBCON as *mut GPIOPortController,
        Port::C => GPCCON as *mut GPIOPortController,
        Port::D => GPDCON as *mut GPIOPortController,
        Port::E => GPECON as *mut GPIOPortController,
        Port::F => GPFCON as *mut GPIOPortController,
        Port::G => GPGCON as *mut GPIOPortController,
        Port::H => GPHCON as *mut GPIOPortController,
        Port::J => GPJCON as *mut GPIOPortController,
    };

    unsafe { &mut (*controller_ptr) }
}
