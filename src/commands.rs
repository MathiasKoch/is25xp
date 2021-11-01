use stm32l4xx_hal::qspi::{QspiMode, QspiReadCommand, QspiWriteCommand};

pub const QPI_ENABLE: QspiWriteCommand = QspiWriteCommand {
    instruction: Some((0x35, QspiMode::SingleChannel)),
    address: None,
    alternative_bytes: None,
    dummy_cycles: 0,
    data: None,
    double_data_rate: false,
};

pub const WRITE_STATUS: QspiWriteCommand = QspiWriteCommand {
    instruction: Some((0x01, QspiMode::SingleChannel)),
    address: None,
    alternative_bytes: None,
    dummy_cycles: 0,
    data: None,
    double_data_rate: false,
};

pub const WRITE_ENABLE: QspiWriteCommand = QspiWriteCommand {
    instruction: Some((0x06, QspiMode::SingleChannel)),
    address: None,
    alternative_bytes: None,
    dummy_cycles: 0,
    data: None,
    double_data_rate: false,
};

pub const GET_STATUS: QspiReadCommand = QspiReadCommand {
    instruction: Some((0x05, QspiMode::SingleChannel)),
    address: None,
    alternative_bytes: None,
    dummy_cycles: 0,
    data_mode: QspiMode::SingleChannel,
    receive_length: 1,
    double_data_rate: false,
};

pub const WRITE: QspiWriteCommand = QspiWriteCommand {
    instruction: Some((0x02, QspiMode::SingleChannel)),
    address: Some((0x8, QspiMode::SingleChannel)),
    alternative_bytes: None,
    dummy_cycles: 0,
    data: None,
    double_data_rate: false,
};

pub const QUAD_WRITE: QspiWriteCommand = QspiWriteCommand {
    instruction: Some((0x32, QspiMode::SingleChannel)),
    address: Some((0x8, QspiMode::SingleChannel)),
    alternative_bytes: None,
    dummy_cycles: 0,
    data: Some((&[0], QspiMode::QuadChannel)),
    double_data_rate: false,
};

pub const READ: QspiReadCommand = QspiReadCommand {
    instruction: Some((0x0b, QspiMode::SingleChannel)),
    address: Some((0x6, QspiMode::SingleChannel)),
    alternative_bytes: None,
    dummy_cycles: 8,
    data_mode: QspiMode::SingleChannel,
    receive_length: 0,
    double_data_rate: false,
};

pub const QUAD_READ: QspiReadCommand = QspiReadCommand {
    instruction: Some((0xEB, QspiMode::SingleChannel)),
    address: Some((0x0, QspiMode::QuadChannel)),
    alternative_bytes: None,
    dummy_cycles: 6,
    data_mode: QspiMode::QuadChannel,
    receive_length: 0,
    double_data_rate: false,
};

pub const ERASE_CHIP: QspiWriteCommand = QspiWriteCommand {
    instruction: Some((0xC7, QspiMode::SingleChannel)),
    address: None,
    alternative_bytes: None,
    dummy_cycles: 0,
    data: None,
    double_data_rate: false,
};

pub const ERASE_BLOCK: QspiWriteCommand = QspiWriteCommand {
    instruction: Some((0xD8, QspiMode::SingleChannel)),
    address: Some((0, QspiMode::SingleChannel)),
    alternative_bytes: None,
    dummy_cycles: 0,
    data: None,
    double_data_rate: false,
};

pub const ERASE_HALF_BLOCK: QspiWriteCommand = QspiWriteCommand {
    instruction: Some((0x52, QspiMode::SingleChannel)),
    address: Some((0, QspiMode::SingleChannel)),
    alternative_bytes: None,
    dummy_cycles: 0,
    data: None,
    double_data_rate: false,
};

pub const ERASE_SECTOR: QspiWriteCommand = QspiWriteCommand {
    instruction: Some((0xD7, QspiMode::SingleChannel)),
    address: Some((0, QspiMode::SingleChannel)),
    alternative_bytes: None,
    dummy_cycles: 0,
    data: None,
    double_data_rate: false,
};
