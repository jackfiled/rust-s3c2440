use crate::clock::{ClockController, ClockStatus};
use crate::utils::Register;
use core::ops::Deref;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveFormatKind {
    Unknown = 0x0000,
    Pcm = 0x0001,
    AdPcm = 0x0002,
    IeeeFloat = 0x0003,
    IbmCvsd = 0x0005,
    Alaw = 0x0006,
    Mulaw = 0x0007,
    OkiAdPcm = 0x0010,
    /// Note: same as ImaAdPcm
    DviAdPcm = 0x0011,
    MediaSpaceAdPcm = 0x0012,
    SierraAdPcm = 0x0013,
    G723AdPcm = 0x0014,
    Digistd = 0x0015,
    Digifix = 0x0016,
    DialogicOkiAdPcm = 0x0017,
    MediasionAdPcm = 0x0018,
    YamahaAdPcm = 0x0020,
    Sonarc = 0x0021,
    DspgroupTruespeech = 0x0022,
    EchosC1 = 0x0023,
    AudiofileAf36 = 0x0024,
    APTX = 0x0025,
    AudiofileAf10 = 0x0026,
    DolbyAc2 = 0x0030,
    Gsm610 = 0x0031,
    MsnAudio = 0x0032,
    AntexAdPcmE = 0x0033,
    ControlResVqlpc = 0x0034,
    Digireal = 0x0035,
    Digiadpcm = 0x0036,
    ControlResCr10 = 0x0037,
    NmsVbxadpcm = 0x0038,
    CsImaadpcm = 0x0039,
    EchosC3 = 0x003A,
    RockwellAdPcm = 0x003B,
    RockwellDigitalK = 0x003C,
    Xebec = 0x003D,
    G721AdPcm = 0x0040,
    G728Celp = 0x0041,
    Mpeg = 0x0050,
    MpegLayer3 = 0x0055,
    Cirrus = 0x0060,
    Espcm = 0x0061,
    Voxware = 0x0062,
    CanopusAtrac = 0x0063,
    G726AdPcm = 0x0064,
    G722AdPcm = 0x0065,
    Dsat = 0x0066,
    DsatDisplay = 0x0067,
    Softsound = 0x0080,
    RhetorexAdPcm = 0x0100,
    CreativeAdPcm = 0x0200,
    CreativeFastSpeech8 = 0x0202,
    CreativeFastSpeech10 = 0x0203,
    Quarterdeck = 0x0220,
    FmTownsSnd = 0x0300,
    BtvDigital = 0x0400,
    Oligsm = 0x1000,
    OliadPcm = 0x1001,
    Olicelp = 0x1002,
    Olisbc = 0x1003,
    Oliopr = 0x1004,
    LhCodec = 0x1100,
    Norris = 0x1400,
    Development = 0xFFFF,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CodecClockKind {
    FS256 = 256,
    FS384 = 384,
}

impl CodecClockKind {
    const KINDS: [CodecClockKind; 2] = [CodecClockKind::FS256, CodecClockKind::FS384];
}

#[repr(C)]
pub struct IisControllerInner {
    control_register: Register,
    mode_register: Register,
    pre_scaler_register: Register,
    fifo_control_register: Register,
    fifo_data_register: Register,
}

const IIS_CONTROLLER_REGISTER: usize = 0x5500_0000;

/// The configuration of IIS bus.
pub struct IisConfig {
    bits_per_sample: u32,
    samples_per_second: u32,
    enable_send: bool,
    enable_receive: bool,
}

impl IisConfig {
    pub fn new(
        bits_per_sample: u32,
        samples_per_second: u32,
        enable_send: bool,
        enable_receive: bool,
    ) -> Self {
        Self {
            bits_per_sample,
            samples_per_second,
            enable_send,
            enable_receive,
        }
    }

    /// The CodecClock and the prescaler factor should be calculated by the samples/s and the
    /// PCLK factor to obtain a frequency closest to the requesting one.
    /// As the document show, the frequency will be PCLK / (r + 1) / N.
    /// Then N can be 256 or 384.
    fn select_codec_clock_and_prescaler(&self, clock: u32) -> (CodecClockKind, u32) {
        let divider_256 = self.calculate_clock_and_prescaler(clock, CodecClockKind::FS256);
        let difference_256 = self
            .samples_per_second
            .abs_diff(clock / (divider_256 + 1) / (CodecClockKind::FS256 as u32));

        let divider_384 = self.calculate_clock_and_prescaler(clock, CodecClockKind::FS384);
        let difference_384 = self
            .samples_per_second
            .abs_diff(clock / (divider_384 + 1) / (CodecClockKind::FS384 as u32));

        if difference_384 < difference_256 {
            (CodecClockKind::FS384, divider_384)
        } else {
            (CodecClockKind::FS256, divider_256)
        }
    }

    fn calculate_clock_and_prescaler(&self, clock: u32, kind: CodecClockKind) -> u32 {
        let target_frequency = self.samples_per_second * (kind as u32);

        // The divider will be a smaller number than the accurate value.
        // So try (divider + 1), which may lead to a more accurate value.
        let divider = clock / target_frequency - 1;

        let small_difference = target_frequency.abs_diff(clock / (divider + 1));
        let bigger_difference = target_frequency.abs_diff(clock / (divider + 2));

        if small_difference > bigger_difference {
            divider + 1
        } else {
            divider
        }
    }
}

pub struct IisHandler<'a> {
    controller: &'a IisController,
    enable_send: bool,
    enable_receive: bool,
}

impl IisHandler<'_> {
    pub fn start(&self) {
        if self.enable_send {
            self.controller.fifo_control_register.set_bit(1, 13);
        }

        if self.enable_receive {
            self.controller.fifo_control_register.set_bit(1, 12);
        }

        // Enable IIS controller.
        self.controller.control_register.set_bit(1, 0);
    }

    pub fn end(&self) {
        // Clear all control registers.
        self.controller.control_register.write(0);
        self.controller.fifo_control_register.write(0);
        self.controller
            .clock_controller
            .close_clock(ClockStatus::IIS);
    }

    #[inline]
    pub fn fifo_address(&self) -> usize {
        self.controller.fifo_data_register.address()
    }
}

pub struct IisController {
    inner: *const IisControllerInner,
    clock_controller: ClockController,
}

impl Deref for IisController {
    type Target = IisControllerInner;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.inner) }
    }
}

impl IisController {
    pub fn new(clock_controller: ClockController) -> Self {
        Self {
            inner: IIS_CONTROLLER_REGISTER as *const IisControllerInner,
            clock_controller,
        }
    }

    pub fn configure(&self, config: &IisConfig, clock: u32) -> IisHandler<'_> {
        // Enable the IIS clock.
        self.clock_controller.open_clock(ClockStatus::IIS);

        let (kind, divider) = config.select_codec_clock_and_prescaler(clock);
        self.pre_scaler_register.write(divider << 5 | divider);

        let iis_mode = (0 << 10) // Use PCLK as input clock.
            | (0 << 8) // Master mode.
            | match (config.enable_send, config.enable_receive) {
                (true, true) => 3 << 6,
                (true, false) => 2 << 6,
                (false, true) => 1 << 6,
                _ => 0
            } // Send or receive mode.
            | 0 << 5 // Master mode.
            | 1 << 4 // MSB format.
            | match config.bits_per_sample {
                8 => 0 << 3,
                16 => 1 << 3,
                _ => unreachable!()
            } // Bits pre samples.
            | match kind {
                CodecClockKind::FS256 => 0 << 2,
                CodecClockKind::FS384 => 1 << 2
            }
            | 1; // Use 32fs as the serial clock as which is most compatible.
        self.mode_register.write(iis_mode);

        let control_mode = 0
            | if config.enable_send { 1 << 5 } else { 0 }
            | if config.enable_receive { 1 << 4 } else { 0 }
            | if !config.enable_send { 1 << 3 } else { 0 }
            | if !config.enable_receive { 1 << 2 } else { 0 }
            | 1 << 1; // Enable prescaler.
        self.control_register.write(control_mode);

        // Enable DMA mode for sending and receiving.
        if config.enable_send {
            self.fifo_control_register.set_bit(1, 15);
        }

        if config.enable_receive {
            self.fifo_control_register.set_bit(1, 14);
        }

        IisHandler {
            controller: self,
            enable_send: config.enable_send,
            enable_receive: config.enable_receive,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Peripheral clock = 52.5MHz.
    const PCLK: u32 = 52_500_000;

    #[test]
    fn codec_kind_tests() {
        assert_eq!(256, CodecClockKind::FS256 as u32);
        assert_eq!(384, CodecClockKind::FS384 as u32);
    }

    #[test]
    fn iis_clock_prescaler_tests() {
        let config = IisConfig {
            bits_per_sample: 8,
            samples_per_second: 8000,
            enable_send: true,
            enable_receive: false,
        };

        // Not the same as the referred code, which is 23/FS256.
        assert_eq!(
            25,
            config.calculate_clock_and_prescaler(PCLK, CodecClockKind::FS256)
        );

        let config = IisConfig {
            samples_per_second: 11025,
            bits_per_sample: 8,
            enable_send: true,
            enable_receive: false,
        };

        assert_eq!(
            11,
            config.calculate_clock_and_prescaler(PCLK, CodecClockKind::FS384)
        );
    }

    #[test]
    fn iis_select_tests() {
        let config = IisConfig {
            bits_per_sample: 8,
            samples_per_second: 8000,
            enable_send: true,
            enable_receive: false,
        };

        assert_eq!(
            (CodecClockKind::FS384, 16),
            config.select_codec_clock_and_prescaler(PCLK)
        );

        let config = IisConfig {
            bits_per_sample: 8,
            samples_per_second: 11025,
            enable_send: true,
            enable_receive: false,
        };

        assert_eq!(
            (CodecClockKind::FS256, 18),
            config.select_codec_clock_and_prescaler(PCLK)
        );

        let config = IisConfig {
            bits_per_sample: 8,
            samples_per_second: 16000,
            enable_send: true,
            enable_receive: false,
        };

        assert_eq!(
            (CodecClockKind::FS256, 12),
            config.select_codec_clock_and_prescaler(PCLK)
        );

        let config = IisConfig {
            bits_per_sample: 8,
            samples_per_second: 44100,
            enable_send: true,
            enable_receive: false,
        };

        assert_eq!(
            (CodecClockKind::FS384, 2),
            config.select_codec_clock_and_prescaler(PCLK)
        );

        let config = IisConfig {
            bits_per_sample: 8,
            samples_per_second: 48000,
            enable_send: true,
            enable_receive: false,
        };

        assert_eq!(
            (CodecClockKind::FS384, 2),
            config.select_codec_clock_and_prescaler(PCLK)
        );
    }
}
