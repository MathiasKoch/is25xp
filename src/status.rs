pub struct Status(u8);

impl From<u8> for Status {
    fn from(v: u8) -> Self {
        Self(v)
    }
}

pub const WIP: u8 = 0x01;
pub const WEL: u8 = 0x02;
pub const QE: u8 = 0x40;
pub const SRWD: u8 = 0x80;

#[allow(dead_code)]
impl Status {
    /// Write In Progress Bit:
    /// - "0" indicates the device is ready (default)
    /// - "1" indicates a write cycle is in progress and the device is busy
    pub fn wip(&self) -> bool {
        self.0 & WIP != 0
    }

    /// Write Enable Latch:
    /// - "0" indicates the device is not write enabled (default)
    /// - "1" indicates the device is write enabled
    pub fn wel(&self) -> bool {
        self.0 & WEL != 0
    }

    /// Block Protection Bit: (See Tables 6.4 for details)
    /// - "0" indicates the specific blocks are not write-protected (default)
    /// - "1" indicates the specific blocks are write-protected
    pub fn bp(&self) {
        todo!()
    }

    /// Quad Enable bit:
    /// - “0” indicates the Quad output function disable (default)
    /// - “1” indicates the Quad output function enable
    pub fn qe(&self) -> bool {
        self.0 & QE != 0
    }

    /// Status Register Write Disable: (See Table 7.1 for details)
    /// - "0" indicates the Status Register is not write-protected (default)
    /// - "1" indicates the Status Register is write-protected
    pub fn srwd(&self) -> bool {
        self.0 & SRWD != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits() {
        assert_eq!(Status(0b00000001).wip(), true);
        assert_eq!(Status(0b00000010).wel(), true);
        assert_eq!(Status(0b01000000).qe(), true);
        assert_eq!(Status(0b10000000).srwd(), true);

        assert_eq!(Status(0b11111111).wip(), true);
        assert_eq!(Status(0b11111111).wel(), true);
        assert_eq!(Status(0b11111111).qe(), true);
        assert_eq!(Status(0b11111111).srwd(), true);

        assert_eq!(Status(0b00000000).wip(), false);
        assert_eq!(Status(0b00000000).wel(), false);
        assert_eq!(Status(0b00000000).qe(), false);
        assert_eq!(Status(0b00000000).srwd(), false);
    }
}
