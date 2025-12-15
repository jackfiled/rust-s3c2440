use crate::gpio::register::{Port, gpio_port_controller};
use crate::gpio::state::{
    Input, InputState, Output, PinFunction, PushPull, UartReceive, UartTransmit,
};
use crate::gpio::{
    CodecClock, IisClock, IisLrSelect, IisSerialDataInput, IisSerialDataOutput, OutputState,
    PinError,
};
use core::marker::PhantomData;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};
use paste::paste;
use seq_macro::seq;

/// A pin in any port.
pub struct Pin<F: PinFunction> {
    port: Port,
    pin: u32,
    _p: PhantomData<F>,
}

/// A pin in port A.
pub struct PortAPin<F: PinFunction> {
    pin: u32,
    _p: PhantomData<F>,
}

impl<F: PinFunction> PortAPin<F> {
    pub fn into_output(self) -> PortAPin<Output<PushPull>> {
        gpio_port_controller(Port::A)
            .control_register
            .set_bit(0, self.pin, 1);
        gpio_port_controller(Port::A)
            .pull_up_register
            .set_bit(1, self.pin, 1);
        PortAPin {
            pin: self.pin,
            _p: PhantomData,
        }
    }

    pub fn erase_port(self) -> Pin<F> {
        Pin {
            port: Port::A,
            pin: self.pin,
            _p: self._p,
        }
    }
}

impl<F: PinFunction> ErrorType for PortAPin<F> {
    type Error = PinError;
}

impl<T: OutputState> OutputPin for PortAPin<Output<T>> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        gpio_port_controller(Port::A)
            .data_register
            .set_bit(0, self.pin, 1);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        gpio_port_controller(Port::A)
            .data_register
            .set_bit(1, self.pin, 1);
        Ok(())
    }
}

seq!(N in 0..=22 {
    pub struct PortAPin~N<F> {
        _p: PhantomData<F>,
    }

    // Port A only supports output.
    impl<F: PinFunction> PortAPin~N<F> {
        pub fn into_output(self) -> PortAPin~N<Output<PushPull>> {
            gpio_port_controller(Port::A)
                .control_register
                .set_bit(0, N, 1);
            gpio_port_controller(Port::A)
                .pull_up_register
                .set_bit(1, N, 1);
            PortAPin~N { _p: PhantomData }
        }

        pub fn erase_pin(self) -> PortAPin<F> {
            PortAPin {
                pin: N,
                _p: self._p,
            }
        }
    }

    impl<F: PinFunction> ErrorType for PortAPin~N<F> {
        type Error = PinError;
    }

    impl PortAPin~N<Output<PushPull>> {
        pub fn init() -> Self {
            gpio_port_controller(Port::A)
                .control_register
                .set_bit(0, N, 1);
            gpio_port_controller(Port::A)
                .pull_up_register
                .set_bit(1, N, 1);
            Self { _p: PhantomData }
        }
    }

    impl<T: OutputState> OutputPin for PortAPin~N<Output<T>> {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            gpio_port_controller(Port::A).data_register.set_bit(0, N, 1);
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            gpio_port_controller(Port::A).data_register.set_bit(1, N, 1);
            Ok(())
        }
    }
});

/// Implement the port definition and pin definition of S3C2440.
/// The macro require the port that:
/// - 2 bits of control_register controls one pin, and b00 means input, b01 means output.
/// - 1 bit of pull_up_register controls one pin, and 0 means enabling, 1 means disabling.
/// - 1 bit of data_register controls one pin, and 0 means off, 1 means on.
/// The macro accept three parameters:
/// - port_name: the uppercase port name, used in port name and Port enumeration.
/// - lower_port_name: the lowercase port name, used to get the `port_` function of controller.
/// - port_number: the number of pin in the port.
macro_rules! impl_port_pin {
    ($port_name: ident, $port_number: literal) => {
        paste!{
            pub struct [<Port $port_name Pin >]<F: PinFunction> {
                pin: u32,
                _p: PhantomData<F>
            }

            impl<F: PinFunction> [<Port $port_name Pin>]<F> {
                pub fn into_input<S: InputState>(self, _state: S) -> [<Port $port_name Pin>]<Input<S>> {
                    gpio_port_controller(Port::$port_name)
                        .control_register
                        .set_bit(0, self.pin * 2, 2);
                    gpio_port_controller(Port::$port_name)
                        .pull_up_register
                        .set_bit(S::pull_up_enable(), self.pin, 1);

                    [<Port $port_name Pin>] {
                        pin: self.pin,
                        _p: PhantomData
                    }
                }

                pub fn into_output(self) -> [<Port $port_name Pin>]<Output<PushPull>> {
                    gpio_port_controller(Port::$port_name)
                        .control_register
                        .set_bit(1, self.pin * 2, 2);
                    gpio_port_controller(Port::$port_name)
                        .pull_up_register
                        .set_bit(1, self.pin, 2);

                    [<Port $port_name Pin>] {
                        pin: self.pin,
                        _p: PhantomData,
                    }
                }

                pub fn erase_port(self) -> Pin<F> {
                    Pin {
                        port: Port::$port_name,
                        pin: self.pin,
                        _p: PhantomData
                    }
                }
            }

            impl<F: PinFunction> ErrorType for [<Port $port_name Pin>]<F> {
                type Error = PinError;
            }

            impl<S: InputState> InputPin for [<Port $port_name Pin>]<Input<S>> {
                fn is_high(&mut self) -> Result<bool, Self::Error> {
                    Ok(gpio_port_controller(Port::$port_name).data_register.is_bit_one(self.pin))
                }

                fn is_low(&mut self) -> Result<bool, Self::Error> {
                    self.is_high().map(|x| !x)
                }
            }

            impl OutputPin for [<Port $port_name Pin>]<Output<PushPull>> {
                fn set_low(&mut self) -> Result<(), Self::Error> {
                    gpio_port_controller(Port::$port_name).data_register.set_bit(1, self.pin, 1);
                    Ok(())
                }

                fn set_high(&mut self) -> Result<(), Self::Error> {
                    gpio_port_controller(Port::$port_name).data_register.set_bit(0, self.pin, 1);
                    Ok(())
                }
            }

            seq!(N in 0..=$port_number {
                pub struct [<Port $port_name Pin>]~N<F: PinFunction> {
                    _p: PhantomData<F>
                }

                impl<F: PinFunction> [<Port $port_name Pin>]~N<F> {
                     pub fn into_input<S: InputState>(self, _state: S) -> [<Port $port_name Pin>]~N<Input<S>> {
                         gpio_port_controller(Port::$port_name)
                             .control_register
                             .set_bit(0, N * 2, 2);
                         gpio_port_controller(Port::$port_name)
                             .pull_up_register
                             .set_bit(S::pull_up_enable(), N, 1);

                         [<Port $port_name Pin>]~N {
                             _p: PhantomData,
                         }
                    }

                    pub fn into_output(self) -> [<Port $port_name Pin>]~N<Output<PushPull>> {
                        gpio_port_controller(Port::$port_name)
                            .control_register
                            .set_bit(1, N * 2, 2);
                        gpio_port_controller(Port::$port_name)
                            .pull_up_register
                            .set_bit(1, N, 1);

                        [<Port $port_name Pin>]~N {
                            _p: PhantomData
                        }
                    }

                    pub fn erase_pin(self) -> [<Port $port_name Pin>]<F> {
                        [<Port $port_name Pin>] {
                            pin: N,
                            _p: PhantomData
                        }
                    }
                }

                impl [<Port $port_name Pin>]~N<Output<PushPull>> {
                    pub fn new() -> Self {
                        let t = Self {
                            _p: PhantomData
                        };

                        t.into_output()
                    }
                }

                impl<F: PinFunction> ErrorType for [<Port $port_name Pin>]~N<F> {
                    type Error = PinError;
                }

                impl<S: InputState> InputPin for [<Port $port_name Pin>]~N<Input<S>> {
                    fn is_high(&mut self) -> Result<bool, Self::Error> {
                        Ok(gpio_port_controller(Port::$port_name).data_register.is_bit_one(N))
                    }

                    fn is_low(&mut self) -> Result<bool, Self::Error> {
                        self.is_high().map(|x| !x)
                    }
                }

                impl OutputPin for [<Port $port_name Pin>]~N<Output<PushPull>> {
                    fn set_high(&mut self) -> Result<(), Self::Error> {
                        gpio_port_controller(Port::$port_name)
                        .data_register.set_bit(1, N, 1);

                        Ok(())
                    }

                    fn set_low(&mut self) -> Result<(), Self::Error> {
                        gpio_port_controller(Port::$port_name)
                            .data_register.set_bit(0, N, 1);
                        Ok(())
                    }
                }
            });
        }
    };
}

impl_port_pin!(B, 10);

impl_port_pin!(C, 15);

impl_port_pin!(D, 15);

impl_port_pin!(E, 15);

impl<F: PinFunction> PortEPin0<F> {
    pub fn into_iis_select(self) -> PortEPin0<IisLrSelect> {
        gpio_port_controller(Port::E)
            .control_register
            .set_bit(2, 0, 2);
        gpio_port_controller(Port::E)
            .pull_up_register
            .set_bit(1, 0, 1);

        PortEPin0 { _p: PhantomData }
    }
}

impl<F: PinFunction> PortEPin1<F> {
    pub fn into_iis_clock(self) -> PortEPin1<IisClock> {
        gpio_port_controller(Port::E)
            .control_register
            .set_bit(2, 2, 2);
        gpio_port_controller(Port::E)
            .pull_up_register
            .set_bit(1, 1, 1);

        PortEPin1 { _p: PhantomData }
    }
}

impl<F: PinFunction> PortEPin2<F> {
    pub fn into_iis_codec_clock(self) -> PortEPin2<CodecClock> {
        gpio_port_controller(Port::E)
            .control_register
            .set_bit(2, 4, 2);
        gpio_port_controller(Port::E)
            .pull_up_register
            .set_bit(1, 2, 1);

        PortEPin2 { _p: PhantomData }
    }
}

impl<F: PinFunction> PortEPin3<F> {
    pub fn into_iis_input(self) -> PortEPin3<IisSerialDataInput> {
        gpio_port_controller(Port::E)
            .control_register
            .set_bit(2, 6, 2);
        gpio_port_controller(Port::E)
            .pull_up_register
            .set_bit(1, 3, 1);

        PortEPin3 { _p: PhantomData }
    }
}

impl<F: PinFunction> PortEPin4<F> {
    pub fn into_iis_output(self) -> PortEPin4<IisSerialDataOutput> {
        gpio_port_controller(Port::E)
            .control_register
            .set_bit(2, 8, 2);
        gpio_port_controller(Port::E)
            .pull_up_register
            .set_bit(1, 4, 1);

        PortEPin4 { _p: PhantomData }
    }
}

impl_port_pin!(F, 7);

impl_port_pin!(G, 15);

impl_port_pin!(H, 10);

impl<F: PinFunction> PortHPin2<F> {
    pub fn into_uart_transmit(self) -> PortHPin2<UartTransmit> {
        gpio_port_controller(Port::H)
            .control_register
            .set_bit(2, 2 * 2, 2);
        gpio_port_controller(Port::H)
            .pull_up_register
            .set_bit(1, 2, 1);
        PortHPin2 { _p: PhantomData }
    }
}

impl<F: PinFunction> PortHPin3<F> {
    pub fn into_uart_receive(self) -> PortHPin3<UartReceive> {
        gpio_port_controller(Port::H)
            .control_register
            .set_bit(2, 2 * 3, 2);
        gpio_port_controller(Port::H)
            .pull_up_register
            .set_bit(1, 3, 1);
        PortHPin3 { _p: PhantomData }
    }
}

impl<F: PinFunction> PortHPin4<F> {
    pub fn into_uart_transmit(self) -> PortHPin4<UartTransmit> {
        gpio_port_controller(Port::H)
            .control_register
            .set_bit(2, 2 * 4, 2);
        gpio_port_controller(Port::H)
            .pull_up_register
            .set_bit(1, 4, 1);
        PortHPin4 { _p: PhantomData }
    }
}

impl<F: PinFunction> PortHPin5<F> {
    pub fn into_uart_receive(self) -> PortHPin5<UartReceive> {
        gpio_port_controller(Port::H)
            .control_register
            .set_bit(2, 2 * 5, 2);
        gpio_port_controller(Port::H)
            .pull_up_register
            .set_bit(1, 5, 1);
        PortHPin5 { _p: PhantomData }
    }
}

impl<F: PinFunction> PortHPin6<F> {
    pub fn into_uart_transmit(self) -> PortHPin6<UartTransmit> {
        gpio_port_controller(Port::H)
            .control_register
            .set_bit(2, 2 * 6, 2);
        gpio_port_controller(Port::H)
            .pull_up_register
            .set_bit(1, 6, 1);
        PortHPin6 { _p: PhantomData }
    }
}

impl<F: PinFunction> PortHPin7<F> {
    pub fn into_uart_receive(self) -> PortHPin7<UartReceive> {
        gpio_port_controller(Port::H)
            .control_register
            .set_bit(2, 2 * 7, 2);
        gpio_port_controller(Port::H)
            .pull_up_register
            .set_bit(1, 7, 1);
        PortHPin7 { _p: PhantomData }
    }
}

impl_port_pin!(J, 12);
