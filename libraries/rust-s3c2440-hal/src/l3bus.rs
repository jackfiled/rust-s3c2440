use crate::gpio::{Output, Port, PortBPin2, PortBPin3, PortBPin4, PushPull, gpio_port_controller};
use crate::iis::IisClockKind;
use crate::nop;
use crate::utils::{BitValue, Register};
use log::debug;

/// L3 Bus is a special bus used by the UDA1341 codec chip.
/// This bus only contains three lines:
/// - Clock
/// - Data
/// - Mode.
pub struct L3BusController;

const L3C: u32 = 1 << 4;
const L3D: u32 = 1 << 3;
const L3M: u32 = 1 << 2;

impl L3BusController {
    /// UDA11341 Bus Address 000101.
    /// The bus address will be sent by the bit 7 ~ bit2, so the value is 000101xx -> 0x14.
    const BUS_ADDRESS: u8 = 0x14;
    const DATA0_MODE: u8 = 0x0;
    const DATA1_MODE: u8 = 0x1;
    const STATUS_MODE: u8 = 0x2;

    pub fn new(
        _mode: PortBPin2<Output<PushPull>>,
        _data: PortBPin3<Output<PushPull>>,
        _clock: PortBPin4<Output<PushPull>>,
    ) -> Self {
        let controller = Self;

        let port_b = gpio_port_controller(Port::B);
        port_b
            .control_register
            .write(port_b.control_register.read() & !(0x3f << 4) | (0x15 << 4));
        port_b
            .pull_up_register
            .write(port_b.pull_up_register.read() & !(0x7 << 2) | (0x7 << 2));

        port_b
            .data_register
            .write(controller.port_b_data().read() & !(L3M | L3C | L3D) | (L3M | L3C)); //Start condition : L3M=H, L3C=H

        controller
    }

    fn write_address(&mut self, mut address: u8) {
        self.port_b_data()
            .write(self.port_b_data().read() & !(L3D | L3M | L3C) | L3C);
        self.tick();

        for _ in 0..8 {
            if address & 0x1 != 0 {
                *self.port_b_data() &= !L3C; //L3C=L
                *self.port_b_data() |= L3D; //L3D=H
                self.tick();

                *self.port_b_data() |= L3C; //L3C=H
                *self.port_b_data() |= L3D; //L3D=H
                self.tick();
            } else {
                *self.port_b_data() &= !L3C; //L3C=L
                *self.port_b_data() &= !L3D; //L3D=L
                self.tick();

                *self.port_b_data() |= L3C; //L3C=H
                *self.port_b_data() &= !L3D; //L3D=L
                self.tick();
            }

            address >>= 1;
        }

        self.port_b_data()
            .write(self.port_b_data().read() & !(L3M | L3C | L3D) | (L3M | L3C)); //Start condition : L3M=H, L3C=H
    }

    fn write_data(&mut self, mut data: u8, halt: bool) {
        if halt {
            //L3C=H(while tstp, L3 interface halt condition)
            self.port_b_data()
                .write(self.port_b_data().read() & !(L3D | L3M | L3C) | L3C);
            self.tick();
        }
        self.port_b_data()
            .write(self.port_b_data().read() & !(L3M | L3C | L3D) | (L3M | L3C)); //L3M=H(in data transfer mode)
        self.tick();

        //GPB[4:2]=L3C:L3D:L3M
        for _ in 0..8 {
            if (data & 0x1) != 0
            //if data's LSB is 'H'
            {
                *self.port_b_data() &= !L3C; //L3C=L
                *self.port_b_data() |= L3D; //L3D=H
                self.tick();

                *self.port_b_data() |= L3C | L3D; //L3C=H,L3D=H
                self.tick();
            } else
            //If data's LSB is 'L'
            {
                *self.port_b_data() &= !L3C; //L3C=L
                *self.port_b_data() &= !L3D; //L3D=L
                self.tick();

                *self.port_b_data() |= L3C; //L3C=H
                *self.port_b_data() &= !L3D; //L3D=L
                self.tick();
            }
            data >>= 1; //For check next bit
        }

        self.port_b_data()
            .write(self.port_b_data().read() & !(L3M | L3C | L3D) | (L3M | L3C)); //L3M=H,L3C=H
    }

    #[inline]
    fn tick(&self) {
        // Under 210MHz, one cycle will cost about 4ns.
        // So 72 * 4ns = 288ns.
        // And for L3 bus, the cycle is about 500ns, the data/address should be preserved on the
        // bus for 250ns at least. And 72 cycle is suitable for `delay_cycles` function.
        // delay_cycles(72);

        // At last, use reference code.
        for _ in 0..4 {
            nop();
        }
    }

    #[inline(always)]
    fn port_b_data(&self) -> &mut Register {
        &mut gpio_port_controller(Port::B).data_register
    }

    pub fn enter_status_mode(&mut self) -> L3StatusMode<'_> {
        self.write_address(Self::BUS_ADDRESS + Self::STATUS_MODE);

        L3StatusMode {
            controller: self,
            halt: false,
        }
    }

    pub fn enter_data0_mode(&mut self) -> L3Data0Mode<'_> {
        self.write_address(Self::BUS_ADDRESS + Self::DATA0_MODE);

        L3Data0Mode {
            controller: self,
            halt: false,
        }
    }
}

pub struct L3StatusMode<'a> {
    controller: &'a mut L3BusController,
    halt: bool,
}

#[repr(u8)]
pub enum CodecClockKind {
    F512 = 0b00,
    F384 = 0b01,
    F256 = 0b10,
}

impl From<IisClockKind> for CodecClockKind {
    fn from(value: IisClockKind) -> Self {
        match value {
            IisClockKind::FS384 => Self::F384,
            IisClockKind::FS256 => Self::F256,
        }
    }
}

#[repr(u8)]
pub enum DataInputFormat {
    IISFormat = 0b000,
    MSBFormat = 0b100,
}

impl<'a> L3StatusMode<'a> {
    pub fn control_group0(
        &mut self,
        reset: bool,
        clock_setting: CodecClockKind,
        data_format: DataInputFormat,
        filter: bool,
    ) {
        // Format: 0 RST SC1 SC0 IF2 IF1 IF0 FILTER
        let mut data = 0;
        data |= (reset.value() as u8) << 6;
        data |= (clock_setting as u8) << 4;
        data |= (data_format as u8) << 1;
        data |= filter.value() as u8;

        debug!("Writing status data: 0x{data:x}");
        self.controller.write_data(data, self.halt);
        self.halt = true;
    }

    pub fn control_group1(
        &mut self,
        output_gain: bool,
        input_gain: bool,
        adc_polarity: bool,
        dac_polarity: bool,
        double_speed: bool,
        enable_adc: bool,
        enable_dac: bool,
    ) {
        let mut data = 1 << 7;
        data |= output_gain.value() << 6;
        data |= input_gain.value() << 5;
        data |= adc_polarity.value() << 4;
        data |= dac_polarity.value() << 3;
        data |= double_speed.value() << 2;
        data |= enable_adc.value() << 1;
        data |= enable_dac.value();

        debug!("Writing status data: 0x{data:x}");
        self.controller.write_data(data as u8, self.halt);
        self.halt = true;
    }
}

pub struct L3Data0Mode<'a> {
    controller: &'a mut L3BusController,
    halt: bool,
}

impl<'a> L3Data0Mode<'a> {
    pub fn control_volume(&mut self, volume: u8) {
        self.controller.write_data(volume & 0x3f, self.halt);
        self.halt = true;
    }

    pub fn control_bass_treble(&mut self, bass: u8, treble: u8) {
        let mut data = 1 << 6;
        data |= (bass & 0xf) << 2;
        data |= treble & 0x3;

        debug!("Writing status data: 0x{data:x}");
        self.controller.write_data(data, self.halt);
        self.halt = true;
    }

    pub fn control_misc(&mut self, detect_peak: bool, emphasis: u32, mute: bool, mode: u32) {
        let mut data = 1 << 7;
        data |= detect_peak.value() << 5;
        data |= emphasis << 3;
        data |= if mute { 1 << 2 } else { 0 };
        data |= mute.value() << 2;
        data |= mode & 0x3;

        debug!("Writing status data: 0x{data:x}");
        self.controller.write_data(data as u8, self.halt);
        self.halt = true;
    }
}
