use crate::clock::{ClockStatus, ClockToken};
use crate::gpio::{
    CodecClock, IisClock, IisLrSelect, IisSerialDataInput, IisSerialDataOutput, PortEPin0,
    PortEPin1, PortEPin2, PortEPin3, PortEPin4,
};
use crate::nop;
use crate::s3c2440::IIS_CONTROLLER;
use crate::utils::{BitValue, Register};
use core::ops::Deref;
use log::info;

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
pub enum IisClockKind {
    FS256 = 256,
    FS384 = 384,
}

impl IisClockKind {
    const KINDS: [IisClockKind; 2] = [IisClockKind::FS256, IisClockKind::FS384];
}

#[repr(C)]
pub struct IisControllerInner {
    control_register: Register,
    mode_register: Register,
    pre_scaler_register: Register,
    fifo_control_register: Register,
    fifo_data_register: Register,
}

/// The configuration of IIS bus.
pub struct IisConfig {
    bits_per_sample: u32,
    samples_per_second: u32,
    enable_send: bool,
    enable_receive: bool,
    enable_dma: bool,
    enable_msb_format: bool,
}

impl IisConfig {
    pub fn new(
        bits_per_sample: u32,
        samples_per_second: u32,
        enable_send: bool,
        enable_receive: bool,
        enable_dma: bool,
        enable_msb_format: bool,
    ) -> Self {
        Self {
            bits_per_sample,
            samples_per_second,
            enable_send,
            enable_receive,
            enable_dma,
            enable_msb_format,
        }
    }

    /// The CodecClock and the prescaler factor should be calculated by the samples/s and the
    /// PCLK factor to obtain a frequency closest to the requesting one.
    /// As the document show, the frequency will be PCLK / (r + 1) / N.
    /// Then N can be 256 or 384.
    pub fn select_codec_clock_and_prescaler(&self, clock: u32) -> (IisClockKind, u32) {
        let divider_256 = self.calculate_clock_and_prescaler(clock, IisClockKind::FS256);
        let difference_256 = self
            .samples_per_second
            .abs_diff(clock / (divider_256 + 1) / (IisClockKind::FS256 as u32));

        let divider_384 = self.calculate_clock_and_prescaler(clock, IisClockKind::FS384);
        let difference_384 = self
            .samples_per_second
            .abs_diff(clock / (divider_384 + 1) / (IisClockKind::FS384 as u32));

        if difference_384 < difference_256 {
            info!("Codec Clock is set to use 384FS, and divider value is {divider_384}.");
            (IisClockKind::FS384, divider_384)
        } else {
            info!("Codec Clock is set to use 256FS, and divider value is {divider_256}.");
            (IisClockKind::FS256, divider_256)
        }

        // match self.samples_per_second {
        //     8000 => (IisClockKind::FS256, 23),
        //     11025 => (IisClockKind::FS384, 11),
        //     16000 => (IisClockKind::FS256, 11),
        //     22050 => (IisClockKind::FS384, 5),
        //     32000 => (IisClockKind::FS256, 5),
        //     44100 => (IisClockKind::FS384, 2),
        //     48000 => (IisClockKind::FS256, 3),
        //     _ => panic!("Not supported."),
        // }
    }

    fn calculate_clock_and_prescaler(&self, clock: u32, kind: IisClockKind) -> u32 {
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
    enable_dma: bool,
}

impl IisHandler<'_> {
    pub fn start(&self) {
        // Enable IIS controller.
        self.controller.control_register.set_bit(1, 0, 1);
    }

    pub fn wait_for_send(&self) {
        if !self.enable_send {
            panic!("IIS sending is not enabled.");
        }

        while self.controller.control_register.is_bit_one(7) {
            nop();
        }
    }

    pub fn end(&self) {
        // Clear all control registers.
        self.controller.control_register.write(0);
        self.controller.fifo_control_register.write(0);
    }

    #[inline]
    pub fn fifo_register(&self) -> &Register {
        &self.controller.fifo_data_register
    }

    pub fn send_buffer_len(&self) -> u32 {
        let value = self.controller.fifo_control_register.read();
        (value >> 6) & 0x3f
    }

    pub fn list_registers(&self) {
        info!(
            "IIS pre scaler register = 0x{:x}",
            self.controller.pre_scaler_register.read()
        );
        info!(
            "IIS mode register = 0x{:x}",
            self.controller.mode_register.read()
        );
        info!(
            "IIS control register = 0x{:x}",
            self.controller.control_register.read()
        );
        info!(
            "IIS FIFO control register = 0x{:x}",
            self.controller.fifo_control_register.read()
        );
    }
}

pub struct IisController {
    inner: *const IisControllerInner,
    clock_token: ClockToken,
}

impl Deref for IisController {
    type Target = IisControllerInner;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.inner) }
    }
}

impl IisController {
    pub fn new(
        _: PortEPin0<IisLrSelect>,
        _: PortEPin1<IisClock>,
        _: PortEPin2<CodecClock>,
        _: PortEPin3<IisSerialDataInput>,
        _: PortEPin4<IisSerialDataOutput>,
        token: ClockToken,
    ) -> Self {
        assert_eq!(&ClockStatus::IIS, token.status());

        Self {
            inner: IIS_CONTROLLER as *const IisControllerInner,
            clock_token: token,
        }
    }

    pub fn configure(&self, config: &IisConfig, clock: u32) -> IisHandler<'_> {
        let (kind, divider) = config.select_codec_clock_and_prescaler(clock);
        self.pre_scaler_register.write(divider << 5 | divider);

        // For control register
        // Let Rx/Tx idle if not needed.
        // Turn on the prescaler.
        let mut control_mode =
            (!config.enable_send).value() << 3 | (!config.enable_receive).value() << 2 | 1 << 1;

        if config.enable_dma {
            control_mode =
                control_mode | config.enable_send.value() << 5 | config.enable_receive.value() << 4;
        }
        self.control_register.write(control_mode);

        // mode[9]: 0 Use PCLK; mode[8]: 0 Work on master mode.
        // mode[5]: 0 Master mode
        let iis_mode = config.enable_send.value() << 7
            | config.enable_receive.value() << 6
            | config.enable_msb_format.value() << 4 // 0 -> IIS format, 1 -> MBS format.
            | match config.bits_per_sample {
            8 => 0 << 3,
            16 => 1 << 3,
            _ => unreachable!()
        } // Bits pre samples.
            | match kind {
            IisClockKind::FS256 => 0 << 2,
            IisClockKind::FS384 => 1 << 2
        }
            | 1; // Use 32fs as the serial clock as which is most compatible.
        self.mode_register.write(iis_mode);

        let mut fifo_control_value = 0;

        // DMA channel.
        fifo_control_value |= (config.enable_dma && config.enable_send).value() << 15;
        fifo_control_value |= (config.enable_dma && config.enable_receive).value() << 14;

        // FIFO switch.
        fifo_control_value |= config.enable_send.value() << 13;
        fifo_control_value |= config.enable_receive.value() << 12;

        // Configure the FIFO control register.
        self.fifo_control_register.write(fifo_control_value);

        IisHandler {
            controller: self,
            enable_dma: config.enable_dma,
            enable_send: config.enable_send,
            enable_receive: config.enable_receive,
        }
    }

    #[inline]
    pub fn fifo_address(&self) -> usize {
        self.fifo_data_register.address()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Peripheral clock = 52.5MHz.
    const PCLK: u32 = 52_500_000;

    #[test]
    fn codec_kind_tests() {
        assert_eq!(256, IisClockKind::FS256 as u32);
        assert_eq!(384, IisClockKind::FS384 as u32);
    }

    #[test]
    fn iis_clock_prescaler_tests() {
        let config = IisConfig {
            bits_per_sample: 8,
            samples_per_second: 8000,
            enable_send: true,
            enable_receive: false,
            enable_dma: false,
            enable_msb_format: false,
        };

        // Not the same as the referred code, which is 23/FS256.
        assert_eq!(
            25,
            config.calculate_clock_and_prescaler(PCLK, IisClockKind::FS256)
        );

        let config = IisConfig {
            samples_per_second: 11025,
            bits_per_sample: 8,
            enable_send: true,
            enable_receive: false,
            enable_dma: false,
            enable_msb_format: false,
        };

        assert_eq!(
            11,
            config.calculate_clock_and_prescaler(PCLK, IisClockKind::FS384)
        );
    }

    #[test]
    fn iis_select_tests() {
        let config = IisConfig {
            bits_per_sample: 8,
            samples_per_second: 8000,
            enable_send: true,
            enable_receive: false,
            enable_dma: false,
            enable_msb_format: false,
        };

        assert_eq!(
            (IisClockKind::FS384, 16),
            config.select_codec_clock_and_prescaler(PCLK)
        );

        let config = IisConfig {
            bits_per_sample: 8,
            samples_per_second: 11025,
            enable_send: true,
            enable_receive: false,
            enable_dma: false,
            enable_msb_format: false,
        };

        assert_eq!(
            (IisClockKind::FS256, 18),
            config.select_codec_clock_and_prescaler(PCLK)
        );

        let config = IisConfig {
            bits_per_sample: 8,
            samples_per_second: 16000,
            enable_send: true,
            enable_receive: false,
            enable_dma: false,
            enable_msb_format: false,
        };

        assert_eq!(
            (IisClockKind::FS256, 12),
            config.select_codec_clock_and_prescaler(PCLK)
        );

        let config = IisConfig {
            bits_per_sample: 16,
            samples_per_second: 22050,
            enable_send: true,
            enable_receive: false,
            enable_dma: false,
            enable_msb_format: false,
        };

        assert_eq!(
            (IisClockKind::FS256, 8),
            config.select_codec_clock_and_prescaler(PCLK)
        );

        let config = IisConfig {
            bits_per_sample: 8,
            samples_per_second: 44100,
            enable_send: true,
            enable_receive: false,
            enable_dma: false,
            enable_msb_format: false,
        };

        assert_eq!(
            (IisClockKind::FS384, 2),
            config.select_codec_clock_and_prescaler(PCLK)
        );

        let config = IisConfig {
            bits_per_sample: 8,
            samples_per_second: 48000,
            enable_send: true,
            enable_receive: false,
            enable_dma: false,
            enable_msb_format: false,
        };

        assert_eq!(
            (IisClockKind::FS384, 2),
            config.select_codec_clock_and_prescaler(PCLK)
        );
    }
}
