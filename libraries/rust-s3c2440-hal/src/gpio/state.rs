use crate::gpio::state::sealed::Sealed;
use core::marker::PhantomData;

mod sealed {
    pub trait Sealed {}
}

/// Pin function for a GPIO pin.
/// For some of S3C2440 GPIO, they can be configured as input, output, external interrupt or bus port.
pub trait PinFunction: Sealed {}

/// Trait indicating that a pin is in input mode.
pub trait InputState: Sealed {
    fn pull_up_enable() -> u32;
}

/// Trait indicating that a pin is in output mode.
pub trait OutputState: Sealed {}

/// A 'container' struct indicating a pin is in output mode.
pub struct Output<S: OutputState> {
    _p: PhantomData<S>,
}

impl<S: OutputState> PinFunction for Output<S> {}

impl<S: OutputState> Sealed for Output<S> {}

/// A 'container' struct indicating a pin in input mode.
pub struct Input<S: InputState> {
    _p: PhantomData<S>,
}

impl<S: InputState> PinFunction for Input<S> {}

impl<S: InputState> Sealed for Input<S> {}

/// Indicating a pin is in normal input mode.
pub struct NormalInput {}

impl InputState for NormalInput {
    fn pull_up_enable() -> u32 {
        0
    }
}

impl Sealed for NormalInput {}

/// Indicating a pin is pulled up.
pub struct PullUp {}

impl InputState for PullUp {
    fn pull_up_enable() -> u32 {
        1
    }
}
impl OutputState for PullUp {}
impl Sealed for PullUp {}

/// Indicating a pin is in push-pull output mode, which is the only output mode S3C2440 supporting.
pub struct PushPull {}

impl OutputState for PushPull {}
impl Sealed for PushPull {}

/// Indicating a pin is used as UART TX port.
pub struct UartTransmit {}
/// Indicating a pin is used as UART RX port.
pub struct UartReceive {}

/// IIS left/right select.
pub struct IisLrSelect {}
/// IIS Clock.
pub struct IisClock {}
/// IIS serial data input.
pub struct IisSerialDataInput {}
/// IIS serial data output.
pub struct IisSerialDataOutput {}
/// IIS codec clock.
pub struct CodecClock {}

macro_rules! impl_pin_function {
    ($($struct_name:ident),+ $(,)?) => {
        $(
            impl $crate::gpio::state::sealed::Sealed for $struct_name {}
            impl $crate::gpio::state::PinFunction for $struct_name {}
        )+
    };
}

impl_pin_function!(
    UartTransmit,
    UartReceive,
    IisLrSelect,
    IisClock,
    IisSerialDataInput,
    IisSerialDataOutput,
    CodecClock
);
