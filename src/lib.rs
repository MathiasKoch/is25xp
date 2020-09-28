//! External flash driver for IS25xP family
#![no_std]

pub mod commands;

// FIXME: Embedded-hal QSPI traits?
use embedded_hal::storage::{Address, BitSubset, IterableByOverlaps, ReadWrite, Region};
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
    Unknown,
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
        let mut sr_arr: [u8; 1] = [0];
        // flash.qspi.transfer(commands::GET_STATUS, &mut sr_arr)?;

        // Set quad enable bit
        sr_arr[0] = sr_arr[0] | 0x40;
        flash
            .qspi
            .write(commands::WRITE_STATUS.data(&sr_arr, QspiMode::SingleChannel))
            .map_err(|_| Error::Qspi)?;

        // Apply QPI mode - This feature does not work..
        // flash.qspi.write(commands::QPI_ENABLE)?;
        // flash
        //     .qspi
        //     .apply_config(flash.qspi.get_config().qpi_mode(true));

        Ok(flash)
    }

    fn is_busy(&self) -> Result<bool, Error> {
        let mut sr_arr: [u8; 1] = [1];
        self.qspi
            .transfer(commands::GET_STATUS, &mut sr_arr)
            .map_err(|_| Error::Qspi)?;

        Ok(sr_arr[0] & 1 == 1)
    }

    fn wait_busy(&self) -> Result<(), Error> {
        while self.is_busy()? {}
        Ok(())
    }

    fn read_native(&self, address: Address, data: &mut [u8]) -> Result<(), Error> {
        self.wait_busy()?;

        self.qspi
            .transfer(
                commands::QUAD_READ
                    .address(address.0 as u32, QspiMode::QuadChannel)
                    .receive_length(data.len() as u32),
                data,
            )
            .map_err(|_| Error::Qspi)
    }

    fn write_page(&self, page: &Page, data: &[u8], address: Address) -> Result<(), Error> {
        if self.is_busy()? {
            return Err(Error::Busy);
        }

        if data.len() > 256 {
            return Err(Error::Unknown);
        }

        self.qspi
            .write(commands::WRITE_ENABLE)
            .map_err(|_| Error::Qspi)?;
        self.qspi
            .write(
                commands::QUAD_WRITE
                    .address(address.0 as u32, QspiMode::SingleChannel)
                    .data(data, QspiMode::QuadChannel),
            )
            .map_err(|_| Error::Qspi)?;

        self.wait_busy()
    }

    fn erase_sector(&self, sector: &Sector) -> Result<(), Error> {
        if self.is_busy()? {
            return Err(Error::Busy);
        }

        self.qspi
            .write(commands::WRITE_ENABLE)
            .map_err(|_| Error::Qspi)?;
        self.qspi
            .write(
                commands::ERASE_SECTOR.address(sector.location().0 as u32, QspiMode::SingleChannel),
            )
            .map_err(|_| Error::Qspi)?;
        self.wait_busy()
    }

    fn erase_halfblock(&self, half_block: &HalfBlock) -> Result<(), Error> {
        if self.is_busy()? {
            return Err(Error::Busy);
        }

        self.qspi
            .write(commands::WRITE_ENABLE)
            .map_err(|_| Error::Qspi)?;
        self.qspi
            .write(
                commands::ERASE_HALF_BLOCK
                    .address(half_block.location().0 as u32, QspiMode::SingleChannel),
            )
            .map_err(|_| Error::Qspi)?;
        self.wait_busy()
    }

    fn erase_block(&self, block: &Block) -> Result<(), Error> {
        if self.is_busy()? {
            return Err(Error::Busy);
        }

        self.qspi
            .write(commands::WRITE_ENABLE)
            .map_err(|_| Error::Qspi)?;
        self.qspi
            .write(
                commands::ERASE_BLOCK.address(block.location().0 as u32, QspiMode::SingleChannel),
            )
            .map_err(|_| Error::Qspi)?;
        self.wait_busy()
    }

    fn erase_chip(&self) -> Result<(), Error> {
        Ok(())
    }
}

pub struct MemoryMap {}
pub struct Block(usize);
pub struct HalfBlock(usize);
pub struct Sector(usize);
pub struct Page(usize);

const BASE_ADDRESS: Address = Address(0x0000_0000);
const PAGES_PER_SECTOR: usize = 16;
const SECTORS_PER_BLOCK: usize = 16;
const SECTORS_PER_HALFBLOCK: usize = 8;
const HALFBLOCKS_PER_BLOCK: usize = SECTORS_PER_BLOCK / SECTORS_PER_HALFBLOCK;
const PAGES_PER_BLOCK: usize = PAGES_PER_SECTOR * SECTORS_PER_BLOCK;
const PAGES_PER_HALFBLOCK: usize = PAGES_PER_SECTOR * HALFBLOCKS_PER_BLOCK;
const PAGE_SIZE: usize = 256;
const SECTOR_SIZE: usize = PAGE_SIZE * PAGES_PER_SECTOR;
const HALFBLOCK_SIZE: usize = SECTOR_SIZE * SECTORS_PER_HALFBLOCK;
const BLOCK_SIZE: usize = SECTOR_SIZE * SECTORS_PER_BLOCK;
const MEMORY_SIZE: usize = NUMBER_OF_BLOCKS * BLOCK_SIZE;
const NUMBER_OF_BLOCKS: usize = 256;
const NUMBER_OF_HALFBLOCKS: usize = NUMBER_OF_BLOCKS * HALFBLOCKS_PER_BLOCK;
const NUMBER_OF_SECTORS: usize = NUMBER_OF_BLOCKS * SECTORS_PER_BLOCK;
const NUMBER_OF_PAGES: usize = NUMBER_OF_SECTORS * PAGES_PER_SECTOR;

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
    pub const fn location() -> Address {
        BASE_ADDRESS
    }
    pub const fn end() -> Address {
        Address(BASE_ADDRESS.0 + MEMORY_SIZE as u32)
    }
    pub const fn size() -> usize {
        MEMORY_SIZE
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
    pub fn location(&self) -> Address {
        BASE_ADDRESS + self.0 * Self::size()
    }
    pub fn end(&self) -> Address {
        self.location() + Self::size()
    }
    pub fn at(address: Address) -> Option<Self> {
        MemoryMap::blocks().find(|s| s.contains(address))
    }
    pub const fn size() -> usize {
        SECTOR_SIZE
    }
}

impl HalfBlock {
    pub fn sectors(&self) -> impl Iterator<Item = Sector> {
        ((self.0 * SECTORS_PER_BLOCK)..((1 + self.0) * SECTORS_PER_BLOCK)).map(Sector)
    }
    pub fn pages(&self) -> impl Iterator<Item = Page> {
        ((self.0 * PAGES_PER_HALFBLOCK)..((1 + self.0) * PAGES_PER_HALFBLOCK)).map(Page)
    }
    pub fn location(&self) -> Address {
        BASE_ADDRESS + self.0 * Self::size()
    }
    pub fn end(&self) -> Address {
        self.location() + Self::size()
    }
    pub fn at(address: Address) -> Option<Self> {
        MemoryMap::halfblocks().find(|s| s.contains(address))
    }
    pub const fn size() -> usize {
        SECTOR_SIZE
    }
}

impl Sector {
    pub fn pages(&self) -> impl Iterator<Item = Page> {
        ((self.0 * PAGES_PER_SECTOR)..((1 + self.0) * PAGES_PER_SECTOR)).map(Page)
    }
    pub fn location(&self) -> Address {
        BASE_ADDRESS + self.0 * Self::size()
    }
    pub fn end(&self) -> Address {
        self.location() + Self::size()
    }
    pub fn at(address: Address) -> Option<Self> {
        MemoryMap::sectors().find(|s| s.contains(address))
    }
    pub const fn size() -> usize {
        SECTOR_SIZE
    }
}

impl Page {
    pub fn location(&self) -> Address {
        BASE_ADDRESS + self.0 * Self::size()
    }
    pub fn end(&self) -> Address {
        self.location() + Self::size()
    }
    pub fn at(address: Address) -> Option<Self> {
        MemoryMap::pages().find(|s| s.contains(address))
    }
    pub const fn size() -> usize {
        PAGE_SIZE
    }
}

impl Region for Block {
    fn contains(&self, address: Address) -> bool {
        let start = Address((BLOCK_SIZE * self.0) as u32);
        (address >= start) && (address < start + BLOCK_SIZE)
    }
}

impl Region for HalfBlock {
    fn contains(&self, address: Address) -> bool {
        let start = Address((HALFBLOCK_SIZE * self.0) as u32);
        (address >= start) && (address < start + HALFBLOCK_SIZE)
    }
}

impl Region for Sector {
    fn contains(&self, address: Address) -> bool {
        let start = Address((SECTOR_SIZE * self.0) as u32);
        (address >= start) && (address < start + SECTOR_SIZE)
    }
}

impl Region for Page {
    fn contains(&self, address: Address) -> bool {
        let start = Address((PAGE_SIZE * self.0) as u32);
        (address >= start) && (address < start + PAGE_SIZE)
    }
}

impl<Q: Qspi> ReadWrite for IS25xP<Q> {
    type Error = Error;

    fn try_read(&mut self, address: Address, bytes: &mut [u8]) -> nb::Result<(), Self::Error> {
        Ok(self.read_native(address, bytes)?)
    }

    fn try_write(&mut self, address: Address, bytes: &[u8]) -> nb::Result<(), Self::Error> {
        for (data, sector, address) in MemoryMap::sectors().overlaps(bytes, address) {
            let offset_into_sector = address.0.saturating_sub(sector.location().0) as usize;
            let mut merge_buffer = [0x00u8; SECTOR_SIZE];
            self.try_read(sector.location(), &mut merge_buffer)?;
            if data.is_subset_of(&merge_buffer[offset_into_sector..]) {
                for (data, page, address) in sector.pages().overlaps(data, address) {
                    self.write_page(&page, data, address)?;
                }
            } else {
                self.erase_sector(&sector)?;
                merge_buffer
                    .iter_mut()
                    .skip(offset_into_sector)
                    .zip(data)
                    .for_each(|(byte, input)| *byte = *input);
                for (data, page, address) in
                    sector.pages().overlaps(&merge_buffer, sector.location())
                {
                    self.write_page(&page, data, address)?;
                }
            }
        }

        Ok(())
    }

    fn range(&self) -> (Address, Address) {
        (MemoryMap::location(), MemoryMap::end())
    }

    fn try_erase(&mut self) -> nb::Result<(), Self::Error> {
        Ok(self.erase_chip()?)
    }
}
