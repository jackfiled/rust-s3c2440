use crate::nop;
use crate::utils::Register;
use bitflags::bitflags;
use core::error::Error;
use core::fmt::{Display, Formatter};
use core::ops::Deref;

#[repr(C)]
pub struct NandFlashControllerInner {
    // 0x00
    configure_register: Register,
    control_register: Register,
    command_register: Register,
    address_register: Register,
    // 0x10
    data_register: Register,
    _ecc_registers: [Register; 3],
    // 0x20
    status_register: Register,
}

impl NandFlashControllerInner {
    fn select(&self) -> NandFlashSelectGuard<'_> {
        self.control_register.set_bit(0, 1);
        NandFlashSelectGuard { inner: self }
    }

    #[inline]
    fn clear_signal(&self) {
        self.status_register.set_bit(1, 2);
    }

    #[inline]
    fn write_command(&self, command: FlashCommand) {
        self.command_register.write_u8(command.bits());
    }

    #[inline]
    fn write_address(&self, address: u8) {
        self.address_register.write_u8(address);
    }

    #[inline]
    fn read_data(&self) -> u8 {
        self.data_register.read_u8()
    }

    #[inline]
    fn write_data(&self, data: u8) {
        self.data_register.write_u8(data);
    }

    fn read_nand_id(&self) -> u16 {
        let select = self.select();
        self.clear_signal();

        self.write_command(FlashCommand::READ_ID);
        self.write_address(0x00);

        let mut device_id = (self.read_data() as u16) << 8;
        device_id = device_id | (self.read_data() as u16);
        drop(select);
        device_id
    }
}

const NAND_CONTROLLER: usize = 0x4E00_0000;

pub struct NandFlashControllerBuilder {
    inner: *const NandFlashControllerInner,
}

impl NandFlashControllerBuilder {
    pub fn build() -> NandFlashController {
        let inner = NAND_CONTROLLER as *const NandFlashControllerInner;
        let inner_ref = unsafe { &(*inner) };

        // Configure the controller.
        // For configuration register:
        // CONF[13:12] = TACLS = 1
        // CONF[10:8] = TWRPH0 = 4
        // CONF[6:4] = TWRPH1 = 1
        // CONF[0] controls the bus width. 0 mens 8 bit and 1 means 16 bits.
        inner_ref
            .configure_register
            .write((1 << 12) | (4 << 8) | (1 << 4) | 0);

        // Configure the controller register.
        // Lazy to write, refer the document.
        inner_ref
            .control_register
            .write((1 << 6) | (1 << 5) | (1 << 4) | (1 << 1) | (1 << 0));

        // Wait for tWB.
        for _ in 0..10 {
            nop();
        }

        let nand_id = inner_ref.read_nand_id();

        NandFlashController { inner, nand_id }
    }
}

/// NAND flash controller.
/// The NAND chip is organized with device, block, page 3-level architecture.
/// Each page is divided into data section and spare section.
/// For TQ2440 board, the page size and block size is fixed, so use constant numbers to store them.
/// 1 device contains 2048 blocks, 1 block contains 64 pages and 1 page contains 2k bytes data and
/// 64 bytes spare data.
/// The device contains 2048 * 64 * 2k = 256MB = 2Gb.
/// Refer to the device manual, the address will be sent with in five cycles:
/// 1st cycle: A0 ~ A7
/// 2nd cycle: A8 ~ A11 filled with zero.
/// 3rd cycle: A12 ~ A19
/// 4th cycle: A20 ~ A27
/// 5th cycle: A28 filled with zero.
/// For the address bits, the following rules apply:
/// A0 ~ A11: the column address in the page, 12 bits.
/// A12 ~ A17: the page address in the block, 6 bits.
/// A18: plane address(for multiplane operations)/block address(for normal operations), 1 bit.
/// A19 ~ A28: the block address, 10 bits.
pub struct NandFlashController {
    nand_id: u16,
    inner: *const NandFlashControllerInner,
}

impl NandFlashController {
    /// Page size, 2KB.
    pub const PAGE_SIZE: usize = 0x800;
    /// Spare size, 64B.
    pub const SPARE_SIZE: usize = 64;
    /// Block size, 128KB, 64 pages.
    pub const BLOCK_SIZE: usize = 0x20000;
    pub const BLOCK_NUM: usize = 2048;
    /// The bad block tag is the first byte of spare data.
    pub const BAD_BLOCK_TAG_OFFSET: usize = Self::PAGE_SIZE;
}

/// RAII Guard to make sure deselect after selecting.
pub struct NandFlashSelectGuard<'a> {
    inner: &'a NandFlashControllerInner,
}

impl Drop for NandFlashSelectGuard<'_> {
    fn drop(&mut self) {
        self.inner.control_register.set_bit(1, 1);
    }
}

// The TQ2440 comes with two different NAND chips. The entire chip ID should be 5 bytes but the
// heading 2 bytes can help us judge the chip model.
const K9F2G07_NAND: u16 = 0xecda;
const S34ML02G1_NAND: u16 = 0x01da;

bitflags! {
    #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
    /// Flash controller commands.
    pub struct FlashCommand: u8 {
        /// Read production ID.
        const READ_ID = 0x90;
        /// Read status.
        const READ_STATUS = 0x70;
        /// Controller Reset.
        const RESET = 0xff;
        /// Page read period 1
        const READ = 0x00;
        /// Page read period 2, for large page.
        const READ_START = 0x30;
        /// Random read period 1.
        const RAND_READ = 0x05;
        /// Random read period 2.
        const RAND_READ_START = 0xE0;
        /// Page write period 1.
        const WRITE = 0x80;
        /// Page write period 2.
        const WRITE_START = 0x10;
        /// Random page write.
        const RAND_WRITE = 0x85;
        /// Block erase period 1.
        const ERASE_1 = 0x60;
        /// Block erase period 2.
        const ERASE_2 = 0xD0;
    }
}

/// Nand flash controller error.
#[derive(Debug, Copy, Clone)]
pub struct NandControllerError {
    message: &'static str,
}

impl NandControllerError {
    fn new(message: &'static str) -> Self {
        Self { message }
    }
}

impl Display for NandControllerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Nand controller error: {}", self.message)
    }
}

impl Error for NandControllerError {}

pub type NandResult<T> = Result<T, NandControllerError>;

/// NandAddress abstraction.
/// For the address bits, the following rules apply:
/// A0 ~ A11: the column address in the page, 12 bits.
/// A12 ~ A17: the page address in the block, 6 bits.
/// A18: plane address(for multiplane operations)/block address(for normal operations), 1 bit.
/// A19 ~ A28: the block address, 10 bits.
#[derive(Copy, Clone)]
pub struct NandAddress(usize);

impl From<usize> for NandAddress {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl NandAddress {
    #[inline]
    fn page_offset(&self) -> usize {
        self.0 % NandFlashController::PAGE_SIZE
    }

    #[inline]
    fn page_number(&self) -> usize {
        self.0 / NandFlashController::PAGE_SIZE
    }

    /// The start address of the block where the address located.
    #[inline]
    fn start_block_address(&self) -> Self {
        (self.0 & (!(NandFlashController::BLOCK_SIZE - 1))).into()
    }

    #[inline]
    fn is_page_aligned(&self) -> bool {
        (self.0 & (NandFlashController::PAGE_SIZE - 1)) == 0
    }

    #[inline]
    fn is_block_aligned(&self) -> bool {
        (self.0 & (NandFlashController::BLOCK_SIZE - 1)) == 0
    }
}

impl core::ops::Add<usize> for NandAddress {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        (self.0 + rhs).into()
    }
}

impl core::ops::AddAssign<usize> for NandAddress {
    fn add_assign(&mut self, rhs: usize) {
        *self = (self.0 + rhs).into()
    }
}

impl Deref for NandFlashController {
    type Target = NandFlashControllerInner;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.inner) }
    }
}

impl NandFlashController {
    /// Detect the read&busy signal to change.
    /// The NFSTAT[2] is a bit indicating the signal is changing or not.
    /// Clear the status by setting the bit to 1.
    fn detect_single(&self) {
        loop {
            if (self.status_register.read() & (1 << 2)) != 0 {
                break;
            }
        }
    }

    /// Waiting for the read & busy single.
    /// The NFSTAT[0] bit 0 means the flash is busy and the bit 1 means the flash is ready.
    fn wait_signal(&self) {
        loop {
            if (self.status_register.read() & 1) != 0 {
                break;
            }
        }
    }

    pub fn device_id(&self) -> u16 {
        self.nand_id
    }

    pub fn read_page(
        &self,
        page_number: usize,
        offset: usize,
        buffer: &mut [u8],
    ) -> NandResult<()> {
        // Make sure the requested size is not bigger than the remaining page size.
        if buffer.len() > Self::PAGE_SIZE + Self::SPARE_SIZE - offset {
            return Err(NandControllerError::new("Read out of range."));
        }

        let guard = self.select();
        self.clear_signal();

        // First write the page number to the flash controller.
        self.write_command(FlashCommand::READ);
        self.write_address(0);
        self.write_address(0);
        self.write_address(page_number as u8);
        self.write_address((page_number >> 8) as u8);
        self.write_address((page_number >> 16) as u8);
        self.write_command(FlashCommand::READ_START);

        // After giving the READ_START command, when R&B signal get a rising edge, the controller
        // will the NFSTAT[2] bit to 1. And after getting this signal, we can read the data or
        // send other commands.
        self.detect_single();

        // If not read the whole page, we need to configure the random reading.
        if !(offset == 0 && buffer.len() == Self::PAGE_SIZE) {
            // Write the offset to the flash controller.
            self.write_command(FlashCommand::RAND_READ);
            self.write_address(offset as u8);
            self.write_address((offset >> 8) as u8);
            self.write_command(FlashCommand::RAND_READ_START);
        }

        for c in buffer.iter_mut() {
            *c = self.read_data();
        }

        drop(guard);
        Ok(())
    }

    pub fn write_page(&self, page_number: usize, offset: usize, data: &[u8]) -> NandResult<()> {
        if data.len() >= Self::PAGE_SIZE + Self::SPARE_SIZE - offset {
            return Err(NandControllerError::new("Write out of range."));
        }

        let guard = self.select();
        self.clear_signal();

        // Send page number to flash controller.
        self.write_command(FlashCommand::WRITE);
        self.write_address(0);
        self.write_address(0);
        self.write_address(page_number as u8);
        self.write_address((page_number >> 8) as u8);
        self.write_address((page_number >> 16) as u8);

        if !(offset == 0 && data.len() == Self::PAGE_SIZE) {
            // Start not at the beginning, using the random command.
            self.write_command(FlashCommand::RAND_WRITE);

            // Send offset to flash controller.
            self.write_address(offset as u8);
            self.write_address((offset >> 8) as u8);
        }

        // Send data to flash controller.
        for &c in data {
            self.write_data(c);
        }

        // Send the programming command to flash controller.
        self.write_command(FlashCommand::WRITE_START);

        // After giving the programming command, we should wait for the R&B signal to get a rising
        // edge, which will be detected by the NFSTAT[2] bit.
        self.detect_single();

        // Send status command to check whether writing succeeded or not.
        self.write_command(FlashCommand::READ_STATUS);
        // Bit 1, cache pass/fail; bit 0, pass/fail.
        let status = self.read_data();
        drop(guard);

        if status & 1 == 0 {
            Ok(())
        } else {
            Err(NandControllerError::new("Writing page failed."))
        }
    }

    pub fn erase_block(&self, block_number: usize) -> NandResult<()> {
        let guard = self.select();
        self.clear_signal();

        // Send the block address to the controller.
        self.write_command(FlashCommand::ERASE_1);
        // The block address are A18 ~ A28.
        self.write_address((block_number << 6) as u8);
        self.write_address((block_number >> 2) as u8);
        self.write_address((block_number >> 10) as u8);

        // Start to erase.
        self.write_command(FlashCommand::ERASE_2);

        // Wait to R&B signal.
        self.detect_single();

        // Send status to check erasing successfully or not.
        self.write_command(FlashCommand::READ_STATUS);
        let status = self.read_data();
        drop(guard);

        if (status & 1) != 0 {
            Err(NandControllerError::new("Failed to erase block."))
        } else {
            Ok(())
        }
    }

    pub fn erase_chip(&self) -> NandResult<()> {
        for id in 0..Self::BLOCK_NUM {
            self.erase_block(id)?;
        }

        Ok(())
    }

    /// Read the spare space of the page where provided address located.
    pub fn read_page_spare_space(&self, address: NandAddress, buffer: &mut [u8]) -> NandResult<()> {
        if buffer.len() > Self::SPARE_SIZE {
            return Err(NandControllerError::new(
                "The buffer length is bigger than spare space size(64B).",
            ));
        }

        self.read_page(address.page_number(), Self::PAGE_SIZE, buffer)
    }

    /// Write the spare space of the page where provided address located.
    pub fn write_page_spare_space(&self, address: NandAddress, buffer: &[u8]) -> NandResult<()> {
        if buffer.len() > Self::SPARE_SIZE {
            return Err(NandControllerError::new(
                "The buffer length is bigger than spare space size(64B).",
            ));
        }

        self.write_page(address.page_number(), Self::PAGE_SIZE, buffer)
    }

    /// Judge whether the block where the address located is bad.
    /// The bad tag is the first byte of spare space of first page of this block.
    /// If 0xFF, good block, otherwise bad block.
    pub fn is_bad_block(&self, address: NandAddress) -> bool {
        let mut tag = 0u8;

        if self
            .read_page_spare_space(
                address.start_block_address(),
                core::slice::from_mut(&mut tag),
            )
            .is_err()
        {
            // If reading failed, just consider as bad block.
            return true;
        }

        tag != 0xFF
    }

    pub fn mark_bad_block(&self, address: NandAddress) -> NandResult<()> {
        // Any value other than 0xFF will be considered as bad block tag.
        let tag = 0xEE;

        if self
            .write_page_spare_space(address.start_block_address(), core::slice::from_ref(&tag))
            .is_err()
        {
            Err(NandControllerError::new("Failed to tag bad block."))
        } else {
            Ok(())
        }
    }

    pub fn read(&self, address: NandAddress, buffer: &mut [u8]) -> NandResult<()> {
        if !address.is_page_aligned() {
            return Err(NandControllerError::new(
                "The start address should be page aligned.",
            ));
        }

        let mut left = buffer.len();
        let mut address = address;
        let mut pos = 0;

        while left > 0 {
            // First check whether meeting bad block.
            if address.is_block_aligned() {
                if self.is_bad_block(address) {
                    // Skip this block.
                    address += Self::BLOCK_SIZE;
                    continue;
                }
            }

            // Then read the data.
            if left >= Self::PAGE_SIZE {
                self.read_page(
                    address.page_number(),
                    address.page_offset(),
                    &mut buffer[pos..pos + Self::PAGE_SIZE],
                )?;
            } else {
                self.read_page(
                    address.page_number(),
                    address.page_offset(),
                    &mut buffer[pos..pos + left],
                )?;

                break;
            }

            left -= Self::PAGE_SIZE;
            address += Self::PAGE_SIZE;
            pos += Self::PAGE_SIZE;
        }

        Ok(())
    }

    pub fn write(&self, address: NandAddress, buffer: &[u8]) -> NandResult<()> {
        if !address.is_page_aligned() {
            return Err(NandControllerError::new(
                "The start address should be page aligned.",
            ));
        }

        let mut left = buffer.len();
        let mut address = address;
        let mut pos = 0;

        while left > 0 {
            // First check whether meeting bad block.
            if address.is_block_aligned() {
                if self.is_bad_block(address) {
                    // Skip this block.
                    address += Self::BLOCK_SIZE;
                    continue;
                }
            }

            // Then read the data.
            if left >= Self::PAGE_SIZE {
                self.write_page(
                    address.page_number(),
                    address.page_offset(),
                    &buffer[pos..pos + Self::PAGE_SIZE],
                )?;
            } else {
                self.write_page(
                    address.page_number(),
                    address.page_offset(),
                    &buffer[pos..pos + left],
                )?;

                break;
            }

            left -= Self::PAGE_SIZE;
            address += Self::PAGE_SIZE;
            pos += Self::PAGE_SIZE;
        }

        Ok(())
    }
}
