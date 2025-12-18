use crate::system::register_interrupt;
use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::RefCell;
use log::info;
use rust_s3c2440_hal::clock::ClockToken;
use rust_s3c2440_hal::dma::{
    DmaChannel2Function, DmaConfig, DmaController, DmaMode, DmaServeMode, DmaSize, MemoryLocation,
};
use rust_s3c2440_hal::gpio::{
    PortBPin2, PortBPin3, PortBPin4, PortEPin0, PortEPin1, PortEPin2, PortEPin3, PortEPin4,
};
use rust_s3c2440_hal::iis::{IisConfig, IisController, IisHandler};
use rust_s3c2440_hal::interrupt::InterruptSource;
use rust_s3c2440_hal::l3bus::{CodecClockKind, DataInputFormat, L3BusController};
use rust_s3c2440_hal::s3c2440::PCLK;

pub struct AudioPlayer {
    l3_bus: L3BusController,
    iis_controller: IisController,
}

/// Magic number used for one DMA transfer size.
const BUFFER_SIZE: usize = 16 * 1024;

pub struct AudioPlayerHandler<'a> {
    player: &'a AudioPlayer,
    pub iis_handler: IisHandler<'a>,
    dma_channel: Rc<RefCell<DmaController>>,
}

impl AudioPlayerHandler<'_> {
    pub fn send_buffer_length(&self) -> u32 {
        self.iis_handler.send_buffer_len()
    }
}

pub struct AudioDmaCallback {
    data_buffer: &'static [u8],
    pos: usize,
    dma_channel: Rc<RefCell<DmaController>>,
}

impl AudioPlayer {
    pub fn new(clock_token: ClockToken) -> Self {
        Self {
            l3_bus: L3BusController::new(PortBPin2::init(), PortBPin3::init(), PortBPin4::init()),
            iis_controller: IisController::new(
                PortEPin0::init().into_iis_select(),
                PortEPin1::init().into_iis_clock(),
                PortEPin2::init().into_iis_codec_clock(),
                PortEPin3::init().into_iis_input(),
                PortEPin4::init().into_iis_output(),
                clock_token,
            ),
        }
    }

    pub fn play_wav(&mut self, wav_file: &'static [u8]) -> AudioPlayerHandler<'_> {
        let data_address = &wav_file[0x2C..];
        let iis_config = IisConfig::new(16, 22050, true, false, true, true);

        // 1. Request the DMA channel2.
        info!("Configuring DMA channel 2...");
        let dma_channel = DmaController::request_channel2(
            DmaChannel2Function::IISOutput,
            DmaConfig {
                source_location: MemoryLocation::System,
                source_auto_increment: true,
                target_location: MemoryLocation::Peripheral,
                target_auto_increment: false,
                dma_mode: DmaMode::Handshake,
                enable_interrupt: true,
                enable_burst: false,
                enable_reload: false,
                serve_mode: DmaServeMode::Single,
            },
        );
        let dma_channel = Rc::new(RefCell::new(dma_channel));

        // 2. Use L3 bus to configure codec chip.
        info!("Configuring the codec chip with L3 bus...");
        let (codec_kind, _) = iis_config.select_codec_clock_and_prescaler(PCLK);

        let mut status_config = self.l3_bus.enter_status_mode();
        status_config.control_group0(
            true,
            CodecClockKind::F256,
            DataInputFormat::IISFormat,
            false,
        );

        let mut status_config = self.l3_bus.enter_status_mode();
        status_config.control_group0(false, codec_kind.into(), DataInputFormat::MSBFormat, false);

        let mut status_config = self.l3_bus.enter_status_mode();
        status_config.control_group1(true, false, false, false, false, false, true);

        let mut data_config = self.l3_bus.enter_data0_mode();
        // Volume 0 means 0db.
        data_config.control_volume(0x8);

        // 3. Register a DMA callback.
        let callback = Rc::new(RefCell::new(AudioDmaCallback {
            data_buffer: &wav_file[0x2C..],
            pos: BUFFER_SIZE,
            dma_channel: dma_channel.clone(),
        }));
        let fifo_address = self.iis_controller.fifo_address();
        register_interrupt(
            InterruptSource::Dma2,
            Box::new(move || {
                let current_pos = callback.borrow().pos;
                // The printing in handler may cause to music stuttering.
                // info!(
                //     "Interrupt triggered, sending buffer [{}..{}].",
                //     current_pos,
                //     current_pos + BUFFER_SIZE
                // );
                let next_data_address = callback.borrow().data_buffer
                    [current_pos..current_pos + BUFFER_SIZE]
                    .as_ptr() as usize;
                callback.borrow_mut().dma_channel.borrow_mut().start_dma(
                    next_data_address,
                    fifo_address,
                    DmaSize::B16,
                    BUFFER_SIZE as u32 / 2,
                );
                callback.borrow_mut().pos += BUFFER_SIZE;
            }),
        );

        // 4. Configure the IIS controller.
        let iis_handler = self.iis_controller.configure(&iis_config, PCLK);

        // 5. Start DMA channel.
        info!("Starting DMA transferring...");
        dma_channel.borrow_mut().start_dma(
            data_address.as_ptr() as usize,
            self.iis_controller.fifo_address(),
            DmaSize::B16,
            BUFFER_SIZE as u32 / 2,
        );

        // 6. Enable IIS controller.
        info!("Starting IIS controller...");
        iis_handler.start();

        info!("Music should be playing...");
        AudioPlayerHandler {
            player: self,
            iis_handler,
            dma_channel,
        }
    }
}
