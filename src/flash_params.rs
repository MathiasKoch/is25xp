use embedded_storage::Region;


pub trait FlashParams {
    
}

pub struct JedecAuto;

impl FlashParams for JedecAuto {
    
}

// ------------------------------------


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

pub struct IS25xPParams;

impl FlashParams for IS25xPParams {
    
}