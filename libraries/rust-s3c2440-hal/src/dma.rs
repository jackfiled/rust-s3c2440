use crate::utils::{BitValue, Register};
use core::ops::Deref;

#[repr(u32)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum DmaChannel {
    Channel0,
    Channel1,
    Channel2,
    Channel3,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum DmaChannel0Function {
    External0 = 0,
    Uart0 = 1,
    SDI = 2,
    Timer = 3,
    USB1 = 4,
    IISOutput = 5,
    PCMin = 6,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum DmaChannel1Function {
    External1 = 0,
    Uart1 = 1,
    IISInput = 2,
    SPI0 = 3,
    USB2 = 4,
    PCMOut = 5,
    SDI = 6,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum DmaChannel2Function {
    IISOutput = 0,
    IISInput = 1,
    SDI = 2,
    Timer = 3,
    USB3 = 4,
    PCMin = 5,
    MICIn = 6,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum DmaChannel3Function {
    Uart2 = 0,
    SDI = 1,
    SPI1 = 2,
    Timer = 3,
    USB4 = 4,
    MICIn = 5,
    PCMOut = 6,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum MemoryLocation {
    System = 0,
    Peripheral = 1,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum DmaMode {
    Query = 0,
    Handshake = 1,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum DmaServeMode {
    Single = 0,
    Full = 1,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum DmaSize {
    B8 = 0,
    B16 = 1,
    B32 = 2,
}

pub struct DmaConfig {
    pub source_location: MemoryLocation,
    pub source_auto_increment: bool,
    pub target_location: MemoryLocation,
    pub target_auto_increment: bool,
    pub dma_mode: DmaMode,
    pub enable_interrupt: bool,
    pub enable_burst: bool,
    pub serve_mode: DmaServeMode,
    pub enable_reload: bool,
}

impl DmaConfig {
    fn source_control_value(&self) -> u32 {
        (self.source_location as u32) << 1 | (!self.source_auto_increment).value()
    }

    fn target_control_value(&self) -> u32 {
        (self.target_location as u32) << 1 | (!self.target_auto_increment).value()
    }

    fn control_value(&self, trigger_mode: &DmaTriggerMode, size: DmaSize, count: u32) -> u32 {
        (self.dma_mode as u32) << 31
            | (self.source_location == MemoryLocation::System).value() << 30
            | self.enable_interrupt.value() << 29
            | self.enable_burst.value() << 28
            | (self.serve_mode as u32) << 27
            | match trigger_mode {
                DmaTriggerMode::Hardware(m) => *m << 24 | 1 << 23,
                DmaTriggerMode::Software => 0,
            }
            | (!self.enable_reload).value() << 22
            | (size as u32) << 20
            | (count & 0xfffff)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DmaTriggerMode {
    Hardware(u32),
    Software,
}

const DMA_CONTROLLER_BASE_ADDRESS: usize = 0x4B00_0000;
const DMA_CONTROLLER_OFFSET: usize = 0x40;

#[repr(C)]
pub struct DmaControllerInner {
    source_register: Register,
    source_control_register: Register,
    target_register: Register,
    target_control_register: Register,
    control_register: Register,
    status_register: Register,
    source_status_register: Register,
    target_status_register: Register,
    mask_trigger_register: Register,
}

pub struct DmaHandler<'a> {
    controller: &'a mut DmaController,
}

impl DmaHandler<'_> {
    pub fn stop(&mut self) {
        self.controller.mask_trigger_register.write(1 << 2);
    }
}

impl Deref for DmaHandler<'_> {
    type Target = DmaController;

    fn deref(&self) -> &Self::Target {
        &self.controller
    }
}

pub struct DmaController {
    inner: *const DmaControllerInner,
    trigger_mode: DmaTriggerMode,
    dma_config: DmaConfig,
}

impl Deref for DmaController {
    type Target = DmaControllerInner;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.inner) }
    }
}

impl DmaController {
    pub fn request_channel0(function: DmaChannel0Function, config: DmaConfig) -> DmaController {
        Self::configure_channel(DmaTriggerMode::Hardware(function as u32), config, 0)
    }

    pub fn request_channel1(function: DmaChannel1Function, config: DmaConfig) -> DmaController {
        Self::configure_channel(DmaTriggerMode::Hardware(function as u32), config, 1)
    }

    pub fn request_channel2(function: DmaChannel2Function, config: DmaConfig) -> DmaController {
        Self::configure_channel(DmaTriggerMode::Hardware(function as u32), config, 2)
    }

    pub fn request_channel3(function: DmaChannel3Function, config: DmaConfig) -> DmaController {
        Self::configure_channel(DmaTriggerMode::Hardware(function as u32), config, 3)
    }

    pub fn start_dma(
        &mut self,
        source_address: usize,
        target_address: usize,
        dma_size: DmaSize,
        count: u32,
    ) -> DmaHandler<'_> {
        self.control_register.write(self.dma_config.control_value(
            &self.trigger_mode,
            dma_size,
            count,
        ));
        self.source_control_register
            .write(self.dma_config.source_control_value());
        self.target_control_register
            .write(self.dma_config.target_control_value());

        self.source_register.write(source_address as u32);
        self.target_register.write(target_address as u32);

        let mask_value = match self.trigger_mode {
            DmaTriggerMode::Hardware(_) => 1 << 1,
            DmaTriggerMode::Software => 1 << 1 | 1,
        };
        self.mask_trigger_register.write(mask_value);

        DmaHandler { controller: self }
    }

    pub fn is_busy(&self) -> bool {
        self.status_register.is_bit_one(20)
    }

    pub fn current_count(&self) -> u32 {
        self.status_register.read() & 0xfffff
    }

    pub fn current_source_address(&self) -> usize {
        self.source_status_register.read() as usize
    }

    pub fn current_target_address(&self) -> usize {
        self.target_status_register.read() as usize
    }

    fn configure_channel(mode: DmaTriggerMode, config: DmaConfig, id: usize) -> DmaController {
        let controller = DmaController {
            inner: (DMA_CONTROLLER_BASE_ADDRESS + DMA_CONTROLLER_OFFSET * id)
                as *const DmaControllerInner,
            trigger_mode: mode,
            dma_config: config,
        };

        controller
    }
}
