use crate::gpio::register::GPIOCONTROLLER;
use crate::gpio::state::{
    Input, InputState, Output, PinFunction, PushPull, UartReceive, UartTransmit,
};
use crate::gpio::{OutputState, PinError};
use core::marker::PhantomData;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin};
use seq_macro::seq;

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
        GPIOCONTROLLER
            .port_a()
            .control_register
            .set_bit(0, self.pin);
        GPIOCONTROLLER
            .port_a()
            .pull_up_register
            .set_bit(1, self.pin);
        PortAPin {
            pin: self.pin,
            _p: PhantomData::default(),
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
        GPIOCONTROLLER.port_a().data_register.set_bit(0, self.pin);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        GPIOCONTROLLER.port_a().data_register.set_bit(1, self.pin);
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
            GPIOCONTROLLER
                .port_a()
                .control_register
                .set_bit(0, N);
            GPIOCONTROLLER
                .port_a()
                .pull_up_register
                .set_bit(1, N);
            PortAPin~N { _p: PhantomData::default() }
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
            GPIOCONTROLLER
                .port_a()
                .control_register
                .set_bit(0, N);
            GPIOCONTROLLER
                .port_a()
                .pull_up_register
                .set_bit(1, N);
            Self { _p: PhantomData::default() }
        }
    }

    impl<T: OutputState> OutputPin for PortAPin~N<Output<T>> {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            GPIOCONTROLLER.port_a().data_register.set_bit(0, N);
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            GPIOCONTROLLER.port_a().data_register.set_bit(1, N);
            Ok(())
        }
    }
});

pub struct PortGPin<F: PinFunction> {
    pin: u32,
    _p: PhantomData<F>,
}

pub struct PortHPin<F: PinFunction> {
    pin: u32,
    _p: PhantomData<F>,
}

impl<F: PinFunction> PortHPin<F> {
    pub fn into_input<S: InputState>(self, _state: S) -> PortHPin<Input<S>> {
        GPIOCONTROLLER
            .port_h()
            .control_register
            .set_bit(1, self.pin * 2);
        GPIOCONTROLLER
            .port_h()
            .pull_up_register
            .set_bit(S::pull_up_enable(), self.pin);
        PortHPin {
            pin: self.pin,
            _p: PhantomData::default(),
        }
    }

    pub fn into_output(self) -> PortHPin<Output<PushPull>> {
        GPIOCONTROLLER
            .port_h()
            .control_register
            .set_bit(0, self.pin * 2);
        GPIOCONTROLLER
            .port_h()
            .pull_up_register
            .set_bit(1, self.pin);

        PortHPin {
            pin: self.pin,
            _p: Default::default(),
        }
    }
}

impl<F: PinFunction> ErrorType for PortHPin<F> {
    type Error = PinError;
}

impl<S: InputState> InputPin for PortHPin<Input<S>> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(GPIOCONTROLLER.port_h().data_register.is_bit_one(self.pin))
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.is_high().map(|x| !x)
    }
}

impl<S: OutputState> OutputPin for PortHPin<Output<S>> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        GPIOCONTROLLER.port_h().data_register.set_bit(0, self.pin);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        GPIOCONTROLLER.port_h().data_register.set_bit(1, self.pin);
        Ok(())
    }
}

seq!(N in 0..=10 {
    pub struct PortHPin~N<F: PinFunction> {
        _p: PhantomData<F>,
    }

    impl<F: PinFunction> PortHPin~N<F> {
        pub fn into_input<S: InputState>(self, _state: S) -> PortHPin~N<Input<S>> {
            GPIOCONTROLLER
                .port_h()
                .control_register
                .set_bit(1, N * 2);
            GPIOCONTROLLER
                .port_h()
                .pull_up_register
                .set_bit(S::pull_up_enable(), N);
            PortHPin~N {
                _p: PhantomData::default(),
            }
        }

        pub fn into_output(self) -> PortHPin~N<Output<PushPull>> {
            GPIOCONTROLLER
                .port_h()
                .control_register
                .set_bit(0, N * 2);
            GPIOCONTROLLER
                .port_h()
                .pull_up_register
                .set_bit(1, N);
            PortHPin~N {
                _p: Default::default(),
            }
        }

        pub fn erase_pin(self) -> PortHPin<F> {
            PortHPin {
                pin: N,
                _p: Default::default()
            }
        }
    }

    impl<F: PinFunction> ErrorType for PortHPin~N<F> {
        type Error = PinError;
    }

    impl PortHPin~N<Output<PushPull>> {
        pub fn init() -> Self {
            GPIOCONTROLLER
                .port_h()
                .control_register
                .set_bit(0, N * 2);
            GPIOCONTROLLER
                .port_h()
                .pull_up_register
                .set_bit(1, N);
            PortHPin~N {
                _p: PhantomData::default(),
            }
        }
    }

    impl OutputPin for PortHPin~N<Output<PushPull>> {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            GPIOCONTROLLER
                .port_h()
                .data_register
                .set_bit(0, N);
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            GPIOCONTROLLER
                .port_h()
                .data_register
                .set_bit(1, N);
            Ok(())
        }
    }

    impl<S: InputState> InputPin for PortHPin~N<Input<S>> {
        fn is_high(&mut self) -> Result<bool, Self::Error> {
            Ok(GPIOCONTROLLER.port_h().data_register.is_bit_one(N))
        }

        fn is_low(&mut self) -> Result<bool, Self::Error> {
            self.is_high().map(|x| !x)
        }
    }
});

impl<F: PinFunction> PortHPin2<F> {
    pub fn into_uart_transmit(self) -> PortHPin2<UartTransmit> {
        GPIOCONTROLLER.port_h().control_register.set_bit(2, 2 * 2);
        GPIOCONTROLLER.port_h().pull_up_register.set_bit(1, 2);
        PortHPin2 {
            _p: Default::default(),
        }
    }
}

impl<F: PinFunction> PortHPin3<F> {
    pub fn into_uart_receive(self) -> PortHPin3<UartReceive> {
        GPIOCONTROLLER.port_h().control_register.set_bit(2, 2 * 3);
        GPIOCONTROLLER.port_h().pull_up_register.set_bit(1, 3);
        PortHPin3 {
            _p: Default::default(),
        }
    }
}

impl<F: PinFunction> PortHPin4<F> {
    pub fn into_uart_transmit(self) -> PortHPin4<UartTransmit> {
        GPIOCONTROLLER.port_h().control_register.set_bit(2, 2 * 4);
        GPIOCONTROLLER.port_h().pull_up_register.set_bit(1, 4);
        PortHPin4 {
            _p: Default::default(),
        }
    }
}

impl<F: PinFunction> PortHPin5<F> {
    pub fn into_uart_receive(self) -> PortHPin5<UartReceive> {
        GPIOCONTROLLER.port_h().control_register.set_bit(2, 2 * 5);
        GPIOCONTROLLER.port_h().pull_up_register.set_bit(1, 5);
        PortHPin5 {
            _p: Default::default(),
        }
    }
}

impl<F: PinFunction> PortHPin6<F> {
    pub fn into_uart_transmit(self) -> PortHPin6<UartTransmit> {
        GPIOCONTROLLER.port_h().control_register.set_bit(2, 2 * 6);
        GPIOCONTROLLER.port_h().pull_up_register.set_bit(1, 6);
        PortHPin6 {
            _p: Default::default(),
        }
    }
}

impl<F: PinFunction> PortHPin7<F> {
    pub fn into_uart_receive(self) -> PortHPin7<UartReceive> {
        GPIOCONTROLLER.port_h().control_register.set_bit(2, 2 * 7);
        GPIOCONTROLLER.port_h().pull_up_register.set_bit(1, 7);
        PortHPin7 {
            _p: Default::default(),
        }
    }
}
