use crate::Register;
use seq_macro::seq;

/// GPIO controller registers.
/// Almost all port has three control registers, listed as control, data and pull-up register except
/// GPIO A port.
#[repr(C)]
struct GPIOPortController {
    pub control_register: Register,
    pub data_register: Register,
    pub pull_up_register: Register,
}

const GPACON: usize = 0x56000000; // Port A control

const GPBCON: usize = 0x56000010; // Port B control

const GPCCON: usize = 0x56000020; // Port C control

const GPDCON: usize = 0x56000030; // Port D control

const GPECON: usize = 0x56000040; // Port E control

const GPFCON: usize = 0x56000050; // Port F control

const GPGCON: usize = 0x56000060; // Port G control

const GPHCON: usize = 0x56000070; // Port H control

const GPJCON: usize = 0x560000d0; // Port J control

pub struct GPIOController {
    port_a: *const GPIOPortController,
    port_b: *const GPIOPortController,
    port_c: *const GPIOPortController,
    port_d: *const GPIOPortController,
    port_e: *const GPIOPortController,
    port_f: *const GPIOPortController,
    port_g: *const GPIOPortController,
    /// Port H contains the UART port, so document it here.
    /// Port H contains 11 ports, each 2 bit control 1 port.
    /// For each port, when configured as 00, it is set as input mode and when configured as 01, it
    /// is set as output mode. Both in input/output mode, the input/output data will be in data
    /// register.
    /// When port set to 10, GPH10 = CLKOUT1, GPH9 = CLKOUT0, GPH8 = UEXTCLK, GPH7 = RXD2,
    /// GPH6 = TXD2, GPH5 = RXD1, GPH4 = TXD1, GPH3 = RXD0, GPH2 = TXD0, GPH1 = nRTS0, GPH0 = nCTS0.
    /// 00 is reserved.
    port_h: *const GPIOPortController,
    port_j: *const GPIOPortController,
}

impl GPIOController {
    pub fn new() -> Self {
        Self {
            port_a: GPACON as *const GPIOPortController,
            port_b: GPBCON as *const GPIOPortController,
            port_c: GPCCON as *const GPIOPortController,
            port_d: GPDCON as *const GPIOPortController,
            port_e: GPECON as *const GPIOPortController,
            port_f: GPFCON as *const GPIOPortController,
            port_g: GPGCON as *const GPIOPortController,
            port_h: GPHCON as *const GPIOPortController,
            port_j: GPJCON as *const GPIOPortController,
        }
    }

    /// Initialize GPIO controller, the GPIO port mapping should be configured following datasheet.
    pub fn initialize(&self) {
        // All pull-up resistors are configured closed.
        self.port_a().control_register.write(0x7f_ffff);

        self.port_b().control_register.write(0x04_4555);
        self.port_b().pull_up_register.write(0x7ff);

        self.port_c().control_register.write(0xaaaa_aaaa);
        self.port_c().pull_up_register.write(0xffff);

        self.port_d().control_register.write(0xaaaa_aaaa);
        self.port_d().pull_up_register.write(0xffff);

        self.port_e().control_register.write(0xaaaa_a800);
        self.port_e().pull_up_register.write(0xffff);

        // FIXME: Below code is commented in reference code.
        self.port_f().control_register.write(0x55aa);
        self.port_f().pull_up_register.write(0xff);

        // 0xff95_ffba can be extended to 1111_1111_1001_0101_1111_1111_1011_1010.
        self.port_g().control_register.write(0xff95_ffba);
        self.port_g().pull_up_register.write(0xffff);

        // 0x2a_faaa can be extended to 10_1010_1111_1010_1010_1010;
        self.port_h().control_register.write(0x2a_faaa);
        self.port_h().pull_up_register.write(0x7ff);

        // 0x2a_faaa can be extended to 10_1010_1111_1010_1010_1010;
        self.port_j().control_register.write(0x02aa_aaaa);
        self.port_j().pull_up_register.write(0x1fff);
    }

    seq!(N in 'a'..='h' {
        #[inline]
        fn port_~N(&self) -> &GPIOPortController {
            unsafe { &(*self.port_~N) }
        }
    });

    #[inline]
    fn port_j(&self) -> &GPIOPortController {
        unsafe { &(*self.port_j) }
    }
}
