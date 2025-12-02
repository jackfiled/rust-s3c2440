use crate::delay_cycles;
use crate::gpio::{Output, PortBPin2, PortBPin3, PortBPin4, PushPull};
use embedded_hal::digital::OutputPin;

/// L3 Bus is a special bus used by the UDA1341 codec chip.
/// This bus only contains three lines:
/// - Clock
/// - Data
/// - Mode.
pub struct L3BusController<PM: OutputPin, PD: OutputPin, PC: OutputPin> {
    mode: PM,
    data: PD,
    clock: PC,
}

impl<PM: OutputPin, PD: OutputPin, PC: OutputPin> L3BusController<PM, PD, PC> {
    /// UDA11341 Bus Address 000101.
    /// The bus address will be sent by the bit 7 ~ bit2, so the value is 000101xx -> 0x14.
    const BUS_ADDRESS: u8 = 0x14;
    const DATA0_MODE: u8 = 0x0;
    const DATA1_MODE: u8 = 0x1;
    const STATUS_MODE: u8 = 0x2;

    fn write_address(&mut self, address: u8) {
        // Pull up the clock.
        self.clock.set_high().unwrap();
        self.mode.set_high().unwrap();
        self.tick();

        // Pull down the mode to send an address.
        self.mode.set_low().unwrap();

        // Waiting for setting up.
        self.tick();

        // Start to send 8 bits.
        for i in 0..8u8 {
            // Pull down the clock to send bit.
            self.clock.set_low().unwrap();
            match (address >> i) & 0x1 == 1 {
                true => self.data.set_high().unwrap(),
                false => self.data.set_low().unwrap(),
            }

            self.tick();

            // Pull up the clock.
            self.clock.set_high().unwrap();
            self.tick()
        }

        // Pull up the mode.
        self.mode.set_high().unwrap();
        self.tick();
    }

    fn write_data(&mut self, data: u8, halt: bool) {
        // If we need to send another data after sending a data.
        // We need to pull down and pull up the mode signal to reenter the data mode.

        if halt {
            self.mode.set_low().unwrap();
            self.tick();
            self.mode.set_high().unwrap();
            self.tick();
        }

        // If we just send data after sending an address, we send the 8 bit seamlessly, as we pull
        // up the mode signal in the end of sending address.
        for i in 0..8u8 {
            // Pull down the clock signal.
            self.clock.set_low().unwrap();
            match (data >> i) & 0x1 == 1 {
                true => self.data.set_high().unwrap(),
                false => self.data.set_low().unwrap(),
            }
            self.tick();

            // Pull up the clock signal.
            self.clock.set_high().unwrap();
            self.tick();
        }
    }

    #[inline]
    fn tick(&self) {
        // Under 210MHz, one cycle will cost about 4ns.
        // So 72 * 4ns = 288ns.
        // And for L3 bus, the cycle is about 500ns, the data/address should be preserved on the
        // bus for 250ns at least. And 72 cycle is suitable for `delay_cycles` function.
        delay_cycles(72);
    }

    pub fn enter_status_mode(&mut self) -> L3StatusMode<'_, PM, PD, PC> {
        self.write_address(Self::BUS_ADDRESS + Self::STATUS_MODE);

        L3StatusMode {
            controller: self,
            halt: false,
        }
    }

    pub fn enter_data0_mode(&mut self) -> L3Data0Mode<'_, PM, PD, PC> {
        self.write_address(Self::BUS_ADDRESS + Self::DATA0_MODE);

        L3Data0Mode {
            controller: self,
            halt: false,
        }
    }
}

impl
    L3BusController<
        PortBPin2<Output<PushPull>>,
        PortBPin3<Output<PushPull>>,
        PortBPin4<Output<PushPull>>,
    >
{
    pub fn new(
        b2: PortBPin2<Output<PushPull>>,
        b3: PortBPin3<Output<PushPull>>,
        b4: PortBPin4<Output<PushPull>>,
    ) -> Self {
        Self {
            mode: b2,
            data: b3,
            clock: b4,
        }
    }
}

pub struct L3StatusMode<'a, PM: OutputPin, PD: OutputPin, PC: OutputPin> {
    controller: &'a mut L3BusController<PM, PD, PC>,
    halt: bool,
}

impl<'a, PM: OutputPin, PD: OutputPin, PC: OutputPin> L3StatusMode<'a, PM, PD, PC> {
    pub fn control_group0(&mut self, reset: bool, frequency: u8, data_format: u8, filter: bool) {
        // Format: 0 RST SC1 SC0 IF2 IF1 IF0 FILTER
        let mut data = 0;
        data |= if reset { 1 << 6 } else { 0 };
        data |= frequency << 4;
        data |= data_format << 1;
        data |= if filter { 1 } else { 0 };

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
        power: u8,
    ) {
        let mut data = 1 << 7;
        data |= if output_gain { 1 << 6 } else { 0 };
        data |= if input_gain { 1 << 5 } else { 0 };
        data |= if adc_polarity { 1 << 4 } else { 0 };
        data |= if dac_polarity { 1 << 3 } else { 0 };
        data |= if double_speed { 1 << 2 } else { 0 };
        data |= power & 0x3;

        self.controller.write_data(data, self.halt);
        self.halt = true;
    }
}

pub struct L3Data0Mode<'a, PM: OutputPin, PD: OutputPin, PC: OutputPin> {
    controller: &'a mut L3BusController<PM, PD, PC>,
    halt: bool,
}

impl<'a, PM: OutputPin, PD: OutputPin, PC: OutputPin> L3Data0Mode<'a, PM, PD, PC> {
    pub fn control_volume(&mut self, volume: u8) {
        self.controller.write_data(volume & 0x3f, self.halt);
        self.halt = true;
    }

    pub fn control_bass_treble(&mut self, bass: u8, treble: u8) {
        let mut data = 1 << 6;
        data |= (bass & 0xf) << 2;
        data |= treble & 0x3;

        self.controller.write_data(data, self.halt);
        self.halt = true;
    }

    pub fn control_misc(&mut self, detect_peak: bool, emphasis: u8, mute: bool, mode: u8) {
        let mut data = 1 << 7;
        data |= if detect_peak { 1 << 5 } else { 0 };
        data |= emphasis << 3;
        data |= if mute { 1 << 2 } else { 0 };
        data |= mode & 0x3;

        self.controller.write_data(data, self.halt);
        self.halt = true;
    }
}
