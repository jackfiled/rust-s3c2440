use crate::utils::Register;
use core::ops::Deref;

trait BitValue {
    fn value(&self) -> u32;
}

impl BitValue for bool {
    fn value(&self) -> u32 {
        match self {
            true => 1,
            false => 0,
        }
    }
}

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
    source_address: usize,
    source_location: MemoryLocation,
    source_auto_increment: bool,
    target_address: usize,
    target_location: MemoryLocation,
    target_auto_increment: bool,
    dma_mode: DmaMode,
    enable_interrupt: bool,
    enable_burst: bool,
    serve_mode: DmaServeMode,
    enable_reload: bool,
    size: DmaSize,
    count: u32,
}

impl DmaConfig {
    fn source_control_value(&self) -> u32 {
        (self.source_location as u32) << 1 | (!self.source_auto_increment).value()
    }

    fn target_control_value(&self) -> u32 {
        (self.target_location as u32) << 1 | (!self.target_auto_increment).value()
    }

    fn control_value(&self) -> u32 {
        (self.dma_mode as u32) << 31
            | (self.source_location == MemoryLocation::System).value() << 30
            | self.enable_interrupt.value() << 29
            | self.enable_burst.value() << 28
            | (self.serve_mode as u32) << 27
            | (!self.enable_reload).value() << 22
            | (self.size as u32) << 20
            | (self.count & 0xfffff)
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

pub struct DmaController {
    inner: *const DmaControllerInner,
    trigger_mode: DmaTriggerMode,
}

impl Deref for DmaController {
    type Target = DmaControllerInner;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.inner) }
    }
}

impl DmaController {
    pub fn request_channel0(function: DmaChannel0Function, config: &DmaConfig) -> DmaController {
        Self::configure_channel(DmaTriggerMode::Hardware(function as u32), config, 0)
    }

    pub fn request_channel1(function: DmaChannel1Function, config: &DmaConfig) -> DmaController {
        Self::configure_channel(DmaTriggerMode::Hardware(function as u32), config, 1)
    }

    pub fn request_channel2(function: DmaChannel2Function, config: &DmaConfig) -> DmaController {
        Self::configure_channel(DmaTriggerMode::Hardware(function as u32), config, 2)
    }

    pub fn request_channel3(function: DmaChannel3Function, config: &DmaConfig) -> DmaController {
        Self::configure_channel(DmaTriggerMode::Hardware(function as u32), config, 3)
    }

    pub fn start_dma(&mut self) -> DmaHandler<'_> {
        let mask_value = match self.trigger_mode {
            DmaTriggerMode::Hardware(_) => 1 << 1,
            DmaTriggerMode::Software => 1 << 1 | 1,
        };
        self.mask_trigger_register.write(mask_value);

        DmaHandler { controller: self }
    }

    fn configure_channel(mode: DmaTriggerMode, config: &DmaConfig, id: usize) -> DmaController {
        let control_value = config.control_value()
            | match mode {
                DmaTriggerMode::Hardware(m) => m << 24 | 1 << 23,
                DmaTriggerMode::Software => 0,
            };

        let controller = DmaController {
            inner: (DMA_CONTROLLER_BASE_ADDRESS + DMA_CONTROLLER_OFFSET * id)
                as *const DmaControllerInner,
            trigger_mode: mode,
        };

        controller.control_register.write(control_value);
        controller
            .source_control_register
            .write(config.source_control_value());
        controller
            .target_control_register
            .write(config.target_control_value());
        controller
            .source_register
            .write(config.source_address as u32);
        controller
            .target_register
            .write(config.target_address as u32);

        controller
    }
}
