#[cfg(test)]
mod it_should {
    use std::{cell::RefCell, collections::VecDeque};

    use crate::commands::{
        ERASE_BLOCK, ERASE_CHIP, ERASE_HALF_BLOCK, ERASE_SECTOR, QUAD_WRITE, WRITE_ENABLE,
        WRITE_STATUS,
    };

    use crate::*;

    struct MockQspi {
        write_operations: RefCell<VecDeque<(Option<(u8, QspiMode)>, Option<u32>, Option<usize>)>>,
    }

    impl MockQspi {
        pub fn new() -> Self {
            Self {
                write_operations: RefCell::new(VecDeque::new()),
            }
        }
    }

    impl Qspi for MockQspi {
        type Error = ();

        fn write(&self, cmd: QspiWriteCommand) -> Result<(), Self::Error> {
            self.write_operations.borrow_mut().push_front((
                cmd.instruction,
                cmd.address.map(|a| a.0),
                cmd.data.map(|d| d.0.len()),
            ));
            Ok(())
        }

        fn transfer(&self, _cmd: QspiReadCommand, buf: &mut [u8]) -> Result<(), Self::Error> {
            // Make sure we do not get stuck in `wait_busy` state
            buf[0] = 0;
            Ok(())
        }
    }

    #[test]
    fn have_correct_capacity() {
        let dev = IS25xP::try_new(MockQspi::new()).unwrap();

        let expected = 512 * 32 * 1024;

        assert_eq!(16_777_216, expected);
        assert_eq!(dev.capacity(), expected);
    }

    #[test]
    fn write_one_aligned_partial_block() {
        let mut dev = IS25xP::try_new(MockQspi::new()).unwrap();

        let bytes = [0u8; 15];

        dev.write(0x100, &bytes[..]).unwrap();

        let operations = dev.qspi.write_operations.borrow();
        let expected_operations = [
            (WRITE_STATUS.instruction, None, Some(1)),
            (WRITE_ENABLE.instruction, None, None),
            (QUAD_WRITE.instruction, Some(0x100), Some(bytes.len())),
        ];

        assert_eq!(
            bytes.len(),
            expected_operations
                .iter()
                .filter(|(i, _, _)| i == &QUAD_WRITE.instruction)
                .filter_map(|(_, _, s)| s.as_ref())
                .sum()
        );

        assert_eq!(operations.len(), expected_operations.len());
        for (i, op) in operations.iter().rev().enumerate() {
            assert_eq!(op, &expected_operations[i]);
        }
    }

    #[test]
    fn write_one_aligned_block() {
        let mut dev = IS25xP::try_new(MockQspi::new()).unwrap();

        let bytes = [0u8; 256];

        dev.write(0x100, &bytes[..]).unwrap();

        let operations = dev.qspi.write_operations.borrow();
        let expected_operations = [
            (WRITE_STATUS.instruction, None, Some(1)),
            (WRITE_ENABLE.instruction, None, None),
            (QUAD_WRITE.instruction, Some(0x100), Some(bytes.len())),
        ];

        assert_eq!(
            bytes.len(),
            expected_operations
                .iter()
                .filter(|(i, _, _)| i == &QUAD_WRITE.instruction)
                .filter_map(|(_, _, s)| s.as_ref())
                .sum()
        );

        assert_eq!(operations.len(), expected_operations.len());
        for (i, op) in operations.iter().rev().enumerate() {
            assert_eq!(op, &expected_operations[i]);
        }
    }

    #[test]
    fn write_multiple_aligned_blocks() {
        let mut dev = IS25xP::try_new(MockQspi::new()).unwrap();

        let bytes = [0u8; 1024];
        dev.write(0x100, &bytes[..]).unwrap();

        let operations = dev.qspi.write_operations.borrow();
        let expected_operations = [
            (WRITE_STATUS.instruction, None, Some(1)),
            (WRITE_ENABLE.instruction, None, None),
            (QUAD_WRITE.instruction, Some(0x100), Some(256)),
            (WRITE_ENABLE.instruction, None, None),
            (QUAD_WRITE.instruction, Some(0x200), Some(256)),
            (WRITE_ENABLE.instruction, None, None),
            (QUAD_WRITE.instruction, Some(0x300), Some(256)),
            (WRITE_ENABLE.instruction, None, None),
            (QUAD_WRITE.instruction, Some(0x400), Some(256)),
        ];

        assert_eq!(
            bytes.len(),
            expected_operations
                .iter()
                .filter(|(i, _, _)| i == &QUAD_WRITE.instruction)
                .filter_map(|(_, _, s)| s.as_ref())
                .sum()
        );

        assert_eq!(operations.len(), expected_operations.len());
        for (i, op) in operations.iter().rev().enumerate() {
            assert_eq!(op, &expected_operations[i]);
        }
    }

    #[test]
    fn write_one_unaligned_block() {
        let mut dev = IS25xP::try_new(MockQspi::new()).unwrap();

        let bytes = [0u8; 256];
        dev.write(0x110, &bytes[..]).unwrap();

        let operations = dev.qspi.write_operations.borrow();
        let expected_operations = [
            (WRITE_STATUS.instruction, None, Some(1)),
            (WRITE_ENABLE.instruction, None, None),
            (QUAD_WRITE.instruction, Some(0x110), Some(240)),
            (WRITE_ENABLE.instruction, None, None),
            (QUAD_WRITE.instruction, Some(0x200), Some(16)),
        ];

        assert_eq!(
            bytes.len(),
            expected_operations
                .iter()
                .filter(|(i, _, _)| i == &QUAD_WRITE.instruction)
                .filter_map(|(_, _, s)| s.as_ref())
                .sum()
        );

        assert_eq!(operations.len(), expected_operations.len());
        for (i, op) in operations.iter().rev().enumerate() {
            assert_eq!(op, &expected_operations[i]);
        }
    }

    #[test]
    fn write_multiple_unaligned_blocks() {
        let mut dev = IS25xP::try_new(MockQspi::new()).unwrap();

        let bytes = [0u8; 1024];
        dev.write(0x110, &bytes[..]).unwrap();

        let operations = dev.qspi.write_operations.borrow();
        let expected_operations = [
            (WRITE_STATUS.instruction, None, Some(1)),
            (WRITE_ENABLE.instruction, None, None),
            (QUAD_WRITE.instruction, Some(0x110), Some(240)),
            (WRITE_ENABLE.instruction, None, None),
            (QUAD_WRITE.instruction, Some(0x200), Some(256)),
            (WRITE_ENABLE.instruction, None, None),
            (QUAD_WRITE.instruction, Some(0x300), Some(256)),
            (WRITE_ENABLE.instruction, None, None),
            (QUAD_WRITE.instruction, Some(0x400), Some(256)),
            (WRITE_ENABLE.instruction, None, None),
            (QUAD_WRITE.instruction, Some(0x500), Some(16)),
        ];

        assert_eq!(
            bytes.len(),
            expected_operations
                .iter()
                .filter(|(i, _, _)| i == &QUAD_WRITE.instruction)
                .filter_map(|(_, _, s)| s.as_ref())
                .sum()
        );

        assert_eq!(operations.len(), expected_operations.len());
        for (i, op) in operations.iter().rev().enumerate() {
            assert_eq!(op, &expected_operations[i]);
        }
    }

    #[test]
    fn erase_single_sector() {
        let mut dev = IS25xP::try_new(MockQspi::new()).unwrap();

        dev.erase(0x00, SECTOR_SIZE).unwrap();

        let operations = dev.qspi.write_operations.borrow();
        let expected_operations = [
            (WRITE_STATUS.instruction, None, Some(1)),
            (WRITE_ENABLE.instruction, None, None),
            (ERASE_SECTOR.instruction, Some(0x00), None),
        ];

        assert_eq!(operations.len(), expected_operations.len());
        for (i, op) in operations.iter().rev().enumerate() {
            assert_eq!(op, &expected_operations[i]);
        }
    }

    #[test]
    fn erase_multiple_sectors() {
        let mut dev = IS25xP::try_new(MockQspi::new()).unwrap();

        dev.erase(0x00, SECTOR_SIZE * 4).unwrap();

        let operations = dev.qspi.write_operations.borrow();
        let expected_operations = [
            (WRITE_STATUS.instruction, None, Some(1)),
            (WRITE_ENABLE.instruction, None, None),
            (ERASE_SECTOR.instruction, Some(0x00), None),
            (WRITE_ENABLE.instruction, None, None),
            (ERASE_SECTOR.instruction, Some(0x00 + SECTOR_SIZE), None),
            (WRITE_ENABLE.instruction, None, None),
            (ERASE_SECTOR.instruction, Some(0x00 + SECTOR_SIZE * 2), None),
            (WRITE_ENABLE.instruction, None, None),
            (ERASE_SECTOR.instruction, Some(0x00 + SECTOR_SIZE * 3), None),
        ];

        assert_eq!(operations.len(), expected_operations.len());
        for (i, op) in operations.iter().rev().enumerate() {
            assert_eq!(op, &expected_operations[i]);
        }
    }

    #[test]
    fn erase_single_half_block() {
        let mut dev = IS25xP::try_new(MockQspi::new()).unwrap();

        dev.erase(0x00, HALFBLOCK_SIZE).unwrap();

        let operations = dev.qspi.write_operations.borrow();
        let expected_operations = [
            (WRITE_STATUS.instruction, None, Some(1)),
            (WRITE_ENABLE.instruction, None, None),
            (ERASE_HALF_BLOCK.instruction, Some(0x00), None),
        ];

        assert_eq!(operations.len(), expected_operations.len());
        for (i, op) in operations.iter().rev().enumerate() {
            assert_eq!(op, &expected_operations[i]);
        }
    }

    #[test]
    fn erase_multiple_half_blocks() {
        let mut dev = IS25xP::try_new(MockQspi::new()).unwrap();

        let start = HALFBLOCK_SIZE;
        dev.erase(start, start + HALFBLOCK_SIZE * 2).unwrap();

        let operations = dev.qspi.write_operations.borrow();
        let expected_operations = [
            (WRITE_STATUS.instruction, None, Some(1)),
            (WRITE_ENABLE.instruction, None, None),
            (ERASE_HALF_BLOCK.instruction, Some(start), None),
            (WRITE_ENABLE.instruction, None, None),
            (
                ERASE_HALF_BLOCK.instruction,
                Some(start + HALFBLOCK_SIZE),
                None,
            ),
        ];

        assert_eq!(operations.len(), expected_operations.len());
        for (i, op) in operations.iter().rev().enumerate() {
            assert_eq!(op, &expected_operations[i]);
        }
    }

    #[test]
    fn erase_single_block() {
        let mut dev = IS25xP::try_new(MockQspi::new()).unwrap();

        dev.erase(0x00, BLOCK_SIZE).unwrap();

        let operations = dev.qspi.write_operations.borrow();
        let expected_operations = [
            (WRITE_STATUS.instruction, None, Some(1)),
            (WRITE_ENABLE.instruction, None, None),
            (ERASE_BLOCK.instruction, Some(0x00), None),
        ];

        assert_eq!(operations.len(), expected_operations.len());
        for (i, op) in operations.iter().rev().enumerate() {
            assert_eq!(op, &expected_operations[i]);
        }
    }

    #[test]
    fn erase_multiple_blocks() {
        let mut dev = IS25xP::try_new(MockQspi::new()).unwrap();

        dev.erase(0x00, BLOCK_SIZE * 3).unwrap();

        let operations = dev.qspi.write_operations.borrow();
        let expected_operations = [
            (WRITE_STATUS.instruction, None, Some(1)),
            (WRITE_ENABLE.instruction, None, None),
            (ERASE_BLOCK.instruction, Some(0x00), None),
            (WRITE_ENABLE.instruction, None, None),
            (ERASE_BLOCK.instruction, Some(0x00 + BLOCK_SIZE), None),
            (WRITE_ENABLE.instruction, None, None),
            (ERASE_BLOCK.instruction, Some(0x00 + BLOCK_SIZE * 2), None),
        ];

        assert_eq!(operations.len(), expected_operations.len());
        for (i, op) in operations.iter().rev().enumerate() {
            assert_eq!(op, &expected_operations[i]);
        }
    }

    #[test]
    fn erase_halfblock_block_halfblock() {
        let mut dev = IS25xP::try_new(MockQspi::new()).unwrap();

        let start = HALFBLOCK_SIZE;
        dev.erase(start, start + HALFBLOCK_SIZE + BLOCK_SIZE + HALFBLOCK_SIZE)
            .unwrap();

        let operations = dev.qspi.write_operations.borrow();
        let expected_operations = [
            (WRITE_STATUS.instruction, None, Some(1)),
            (WRITE_ENABLE.instruction, None, None),
            (ERASE_HALF_BLOCK.instruction, Some(start), None),
            (WRITE_ENABLE.instruction, None, None),
            (ERASE_BLOCK.instruction, Some(start + HALFBLOCK_SIZE), None),
            (WRITE_ENABLE.instruction, None, None),
            (
                ERASE_HALF_BLOCK.instruction,
                Some(start + BLOCK_SIZE + HALFBLOCK_SIZE),
                None,
            ),
        ];

        assert_eq!(operations.len(), expected_operations.len());
        for (i, op) in operations.iter().rev().enumerate() {
            assert_eq!(op, &expected_operations[i]);
        }
    }

    #[test]
    fn erase_sector_halfblock_block() {
        let mut dev = IS25xP::try_new(MockQspi::new()).unwrap();

        let start = HALFBLOCK_SIZE - SECTOR_SIZE;
        dev.erase(start, start + SECTOR_SIZE + HALFBLOCK_SIZE + BLOCK_SIZE)
            .unwrap();

        let operations = dev.qspi.write_operations.borrow();
        let expected_operations = [
            (WRITE_STATUS.instruction, None, Some(1)),
            (WRITE_ENABLE.instruction, None, None),
            (ERASE_SECTOR.instruction, Some(start), None),
            (WRITE_ENABLE.instruction, None, None),
            (
                ERASE_HALF_BLOCK.instruction,
                Some(start + SECTOR_SIZE),
                None,
            ),
            (WRITE_ENABLE.instruction, None, None),
            (
                ERASE_BLOCK.instruction,
                Some(start + SECTOR_SIZE + HALFBLOCK_SIZE),
                None,
            ),
        ];

        assert_eq!(operations.len(), expected_operations.len());
        for (i, op) in operations.iter().rev().enumerate() {
            assert_eq!(op, &expected_operations[i]);
        }
    }

    #[test]
    fn erase_chip() {
        let mut dev = IS25xP::try_new(MockQspi::new()).unwrap();

        dev.erase(0x00, MEMORY_SIZE).unwrap();

        let operations = dev.qspi.write_operations.borrow();
        let expected_operations = [
            (WRITE_STATUS.instruction, None, Some(1)),
            (WRITE_ENABLE.instruction, None, None),
            (ERASE_CHIP.instruction, None, None),
        ];

        assert_eq!(operations.len(), expected_operations.len());
        for (i, op) in operations.iter().rev().enumerate() {
            assert_eq!(op, &expected_operations[i]);
        }
    }
}
