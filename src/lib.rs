//! External flash driver for IS25xP family

#![cfg_attr(not(test), no_std)]

pub mod commands;
// mod flash_params;
mod status;

#[cfg(test)]
mod tests;

use embedded_storage::{
    nor_flash::{MultiwriteNorFlash, NorFlash, ReadNorFlash},
    Region,
};
use status::{Status, QE};
use stm32l4xx_hal::{
    pac::QUADSPI,
    qspi::{
        ClkPin, IO0Pin, IO1Pin, IO2Pin, IO3Pin, NCSPin, QspiError, QspiMode, QspiReadCommand,
        QspiWriteCommand,
    },
};

pub trait Qspi {
    type Error: core::fmt::Debug;

    fn write(&self, cmd: QspiWriteCommand) -> Result<(), Self::Error>;
    fn transfer(&self, cmd: QspiReadCommand, buf: &mut [u8]) -> Result<(), Self::Error>;
}

impl<CLK, NCS, IO0, IO1, IO2, IO3> Qspi
    for stm32l4xx_hal::qspi::Qspi<(CLK, NCS, IO0, IO1, IO2, IO3)>
where
    CLK: ClkPin<QUADSPI>,
    NCS: NCSPin<QUADSPI>,
    IO0: IO0Pin<QUADSPI>,
    IO1: IO1Pin<QUADSPI>,
    IO2: IO2Pin<QUADSPI>,
    IO3: IO3Pin<QUADSPI>,
{
    type Error = QspiError;

    fn write(&self, cmd: QspiWriteCommand) -> Result<(), Self::Error> {
        stm32l4xx_hal::qspi::Qspi::write(self, cmd)
    }

    fn transfer(&self, cmd: QspiReadCommand, buf: &mut [u8]) -> Result<(), Self::Error> {
        stm32l4xx_hal::qspi::Qspi::transfer(self, cmd, buf)
    }
}

#[derive(Debug)]
pub enum Error {
    Busy,
    Qspi,
    OutOfBounds,
    Alignment,
    Size,
}

pub struct IS25xP<Q> {
    qspi: Q,
}

impl<Q> IS25xP<Q>
where
    Q: Qspi,
{
    pub fn try_new(qspi: Q) -> Result<Self, Error> {
        let flash = IS25xP { qspi };
        flash.wait_busy()?;
        // Set quad enable bit
        flash
            .qspi
            .write(commands::WRITE_STATUS.data(&[QE], QspiMode::SingleChannel))
            .map_err(|_| Error::Qspi)?;

        // Apply QPI mode - This feature does not work..
        // flash.qspi.write(commands::QPI_ENABLE).map_err(|_| Error::Qspi)?;
        // flash
        //     .qspi
        //     .apply_config(flash.qspi.get_config().qpi_mode(true));

        Ok(flash)
    }

    fn status(&self) -> Result<Status, Error> {
        let mut sr_arr = [1u8; 1];
        self.qspi
            .transfer(commands::GET_STATUS, &mut sr_arr)
            .map_err(|_| Error::Qspi)?;

        Ok(sr_arr[0].into())
    }

    fn wait_busy(&self) -> Result<(), Error> {
        while self.status()?.wip() {}
        Ok(())
    }

    pub fn read_native(&self, offset: u32, data: &mut [u8]) -> Result<(), Error> {
        self.wait_busy()?;

        self.qspi
            .transfer(
                commands::QUAD_READ
                    .address(offset, QspiMode::QuadChannel)
                    .receive_length(data.len() as u32),
                data,
            )
            .map_err(|_| Error::Qspi)
    }

    pub fn write_page(&self, offset: u32, data: &[u8]) -> Result<(), Error> {
        if self.status()?.wip() {
            return Err(Error::Busy);
        }

        if data.len() > PAGE_SIZE as usize {
            return Err(Error::Size);
        }

        self.qspi
            .write(commands::WRITE_ENABLE)
            .map_err(|_| Error::Qspi)?;
        self.qspi
            .write(
                commands::QUAD_WRITE
                    .address(offset, QspiMode::SingleChannel)
                    .data(data, QspiMode::QuadChannel),
            )
            .map_err(|_| Error::Qspi)?;

        self.wait_busy()
    }

    pub fn erase_sector(&self, sector: &Sector) -> Result<(), Error> {
        if self.status()?.wip() {
            return Err(Error::Busy);
        }

        self.qspi
            .write(commands::WRITE_ENABLE)
            .map_err(|_| Error::Qspi)?;
        self.qspi
            .write(commands::ERASE_SECTOR.address(sector.start(), QspiMode::SingleChannel))
            .map_err(|_| Error::Qspi)?;
        self.wait_busy()
    }

    pub fn erase_halfblock(&self, half_block: &HalfBlock) -> Result<(), Error> {
        if self.status()?.wip() {
            return Err(Error::Busy);
        }

        self.qspi
            .write(commands::WRITE_ENABLE)
            .map_err(|_| Error::Qspi)?;
        self.qspi
            .write(commands::ERASE_HALF_BLOCK.address(half_block.start(), QspiMode::SingleChannel))
            .map_err(|_| Error::Qspi)?;
        self.wait_busy()
    }

    pub fn erase_block(&self, block: &Block) -> Result<(), Error> {
        if self.status()?.wip() {
            return Err(Error::Busy);
        }

        self.qspi
            .write(commands::WRITE_ENABLE)
            .map_err(|_| Error::Qspi)?;
        self.qspi
            .write(commands::ERASE_BLOCK.address(block.start(), QspiMode::SingleChannel))
            .map_err(|_| Error::Qspi)?;
        self.wait_busy()
    }

    pub fn erase_chip(&self) -> Result<(), Error> {
        if self.status()?.wip() {
            return Err(Error::Busy);
        }

        self.qspi
            .write(commands::WRITE_ENABLE)
            .map_err(|_| Error::Qspi)?;
        self.qspi
            .write(commands::ERASE_CHIP)
            .map_err(|_| Error::Qspi)?;
        self.wait_busy()
    }
}

pub struct MemoryMap;
pub struct Block(u32);
pub struct HalfBlock(u32);
pub struct Sector(u32);
pub struct Page(u32);

const BASE_ADDRESS: u32 = 0x0000_0000;
const PAGES_PER_SECTOR: u32 = 16;
const SECTORS_PER_BLOCK: u32 = 16;
const SECTORS_PER_HALFBLOCK: u32 = 8;
const HALFBLOCKS_PER_BLOCK: u32 = SECTORS_PER_BLOCK / SECTORS_PER_HALFBLOCK;
const PAGES_PER_BLOCK: u32 = PAGES_PER_SECTOR * SECTORS_PER_BLOCK;
const PAGES_PER_HALFBLOCK: u32 = PAGES_PER_SECTOR * HALFBLOCKS_PER_BLOCK;
const PAGE_SIZE: u32 = 256;
const SECTOR_SIZE: u32 = PAGE_SIZE * PAGES_PER_SECTOR;
const HALFBLOCK_SIZE: u32 = SECTOR_SIZE * SECTORS_PER_HALFBLOCK;
const BLOCK_SIZE: u32 = SECTOR_SIZE * SECTORS_PER_BLOCK;
const MEMORY_SIZE: u32 = NUMBER_OF_BLOCKS * BLOCK_SIZE;
const NUMBER_OF_BLOCKS: u32 = 256;
const NUMBER_OF_HALFBLOCKS: u32 = NUMBER_OF_BLOCKS * HALFBLOCKS_PER_BLOCK;
const NUMBER_OF_SECTORS: u32 = NUMBER_OF_BLOCKS * SECTORS_PER_BLOCK;
const NUMBER_OF_PAGES: u32 = NUMBER_OF_SECTORS * PAGES_PER_SECTOR;

impl MemoryMap {
    pub fn blocks() -> impl Iterator<Item = Block> {
        (0..NUMBER_OF_BLOCKS).map(Block)
    }
    pub fn halfblocks() -> impl Iterator<Item = HalfBlock> {
        (0..NUMBER_OF_HALFBLOCKS).map(HalfBlock)
    }
    pub fn sectors() -> impl Iterator<Item = Sector> {
        (0..NUMBER_OF_SECTORS).map(Sector)
    }
    pub fn pages() -> impl Iterator<Item = Page> {
        (0..NUMBER_OF_PAGES).map(Page)
    }
    pub const fn start() -> u32 {
        BASE_ADDRESS
    }
    pub const fn end() -> u32 {
        BASE_ADDRESS + MEMORY_SIZE as u32
    }
    pub const fn size() -> usize {
        MEMORY_SIZE as usize
    }
}

impl Block {
    pub fn sectors(&self) -> impl Iterator<Item = Sector> {
        ((self.0 * SECTORS_PER_BLOCK)..((1 + self.0) * SECTORS_PER_BLOCK)).map(Sector)
    }
    pub fn halfblocks(&self) -> impl Iterator<Item = HalfBlock> {
        ((self.0 * HALFBLOCKS_PER_BLOCK)..((1 + self.0) * HALFBLOCKS_PER_BLOCK)).map(HalfBlock)
    }
    pub fn pages(&self) -> impl Iterator<Item = Page> {
        ((self.0 * PAGES_PER_BLOCK)..((1 + self.0) * PAGES_PER_BLOCK)).map(Page)
    }
    pub fn start(&self) -> u32 {
        BASE_ADDRESS + self.0 * Self::size() as u32
    }
    pub fn end(&self) -> u32 {
        self.start() + Self::size() as u32
    }
    pub fn at(address: u32) -> Option<Self> {
        MemoryMap::blocks().find(|s| s.contains(address))
    }
    pub const fn size() -> usize {
        BLOCK_SIZE as usize
    }
}

impl HalfBlock {
    pub fn sectors(&self) -> impl Iterator<Item = Sector> {
        ((self.0 * SECTORS_PER_HALFBLOCK)..((1 + self.0) * SECTORS_PER_HALFBLOCK)).map(Sector)
    }
    pub fn pages(&self) -> impl Iterator<Item = Page> {
        ((self.0 * PAGES_PER_HALFBLOCK)..((1 + self.0) * PAGES_PER_HALFBLOCK)).map(Page)
    }
    pub fn start(&self) -> u32 {
        BASE_ADDRESS + self.0 * Self::size() as u32
    }
    pub fn end(&self) -> u32 {
        self.start() + Self::size() as u32
    }
    pub fn at(address: u32) -> Option<Self> {
        MemoryMap::halfblocks().find(|s| s.contains(address))
    }
    pub const fn size() -> usize {
        HALFBLOCK_SIZE as usize
    }
}

impl Sector {
    pub fn pages(&self) -> impl Iterator<Item = Page> {
        ((self.0 * PAGES_PER_SECTOR)..((1 + self.0) * PAGES_PER_SECTOR)).map(Page)
    }
    pub fn start(&self) -> u32 {
        BASE_ADDRESS + self.0 * Self::size() as u32
    }
    pub fn end(&self) -> u32 {
        self.start() + Self::size() as u32
    }
    pub fn at(address: u32) -> Option<Self> {
        MemoryMap::sectors().find(|s| s.contains(address))
    }
    pub const fn size() -> usize {
        SECTOR_SIZE as usize
    }
}

impl Page {
    pub fn start(&self) -> u32 {
        BASE_ADDRESS + self.0 * Self::size() as u32
    }
    pub fn end(&self) -> u32 {
        self.start() + Self::size() as u32
    }
    pub fn at(address: u32) -> Option<Self> {
        MemoryMap::pages().find(|s| s.contains(address))
    }
    pub const fn size() -> usize {
        PAGE_SIZE as usize
    }
}

impl Region for Block {
    fn contains(&self, address: u32) -> bool {
        let start = BLOCK_SIZE * self.0;
        (start <= address) && (address < start + BLOCK_SIZE)
    }
}

impl Region for HalfBlock {
    fn contains(&self, address: u32) -> bool {
        let start = HALFBLOCK_SIZE * self.0;
        (start <= address) && (address < start + HALFBLOCK_SIZE)
    }
}

impl Region for Sector {
    fn contains(&self, address: u32) -> bool {
        let start = SECTOR_SIZE * self.0;
        (start <= address) && (address < start + SECTOR_SIZE)
    }
}

impl Region for Page {
    fn contains(&self, address: u32) -> bool {
        let start = PAGE_SIZE * self.0;
        (start <= address) && (address < start + PAGE_SIZE)
    }
}

impl<Q: Qspi> ReadNorFlash for IS25xP<Q> {
    type Error = Error;

    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        if offset > self.capacity() as u32 {
            return Err(Error::OutOfBounds);
        }

        self.read_native(offset, bytes)
    }

    fn capacity(&self) -> usize {
        MemoryMap::size()
    }
}

impl<Q: Qspi> NorFlash for IS25xP<Q> {
    const WRITE_SIZE: usize = 1;

    const ERASE_SIZE: usize = SECTOR_SIZE as usize;

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        if offset as usize + bytes.len() > self.capacity() {
            return Err(Error::OutOfBounds);
        }

        let mut alignment_offset = 0;
        let mut aligned_address = MemoryMap::start() + offset;

        if offset % PAGE_SIZE != 0 {
            alignment_offset = core::cmp::min(PAGE_SIZE - offset % PAGE_SIZE, bytes.len() as u32);
            self.write_page(aligned_address, &bytes[..alignment_offset as usize])?;

            aligned_address += alignment_offset;
        }

        let mut chunks = bytes[alignment_offset as usize..].chunks_exact(PAGE_SIZE as usize);
        for exact_chunk in &mut chunks {
            self.write_page(aligned_address, exact_chunk)?;
            aligned_address += PAGE_SIZE;
        }

        let remainder = chunks.remainder();
        if !remainder.is_empty() {
            self.write_page(aligned_address, remainder)?;
        }

        Ok(())
    }

    fn erase(&mut self, mut from: u32, to: u32) -> Result<(), Self::Error> {
        // Check that from & to is properly aligned to a proper erase resolution
        if to % Self::ERASE_SIZE as u32 != 0 || from % Self::ERASE_SIZE as u32 != 0 {
            return Err(Error::Alignment);
        }

        // Shortcut to erase entire chip
        if MemoryMap::start() == from && MemoryMap::end() == to {
            return self.erase_chip();
        }

        while from < to {
            if from % BLOCK_SIZE == 0 && from + BLOCK_SIZE <= to {
                let block = Block::at(from).ok_or(Error::OutOfBounds)?;
                self.erase_block(&block)?;
                from += BLOCK_SIZE;
            } else if from % HALFBLOCK_SIZE == 0 && from + HALFBLOCK_SIZE <= to {
                let halfblock = HalfBlock::at(from).ok_or(Error::OutOfBounds)?;
                self.erase_halfblock(&halfblock)?;
                from += HALFBLOCK_SIZE;
            } else {
                let sector = Sector::at(from).ok_or(Error::OutOfBounds)?;
                self.erase_sector(&sector)?;
                from += SECTOR_SIZE;
            }
        }

        Ok(())
    }
}

/// Note: A program operation can alter “1”s into “0”s. The same byte location
/// or page may be programmed more than once, to incrementally change “1”s to
/// “0”s. An erase operation is required to change “0”s to “1”s.
impl<Q: Qspi> MultiwriteNorFlash for IS25xP<Q> {}
