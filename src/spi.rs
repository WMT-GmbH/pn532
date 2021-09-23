use crate::Interface;
use embedded_hal::spi::blocking::{Operation, Transactional};

pub const PN532_SPI_STATREAD: u8 = 0x02;
pub const PN532_SPI_DATAWRITE: u8 = 0x01;
pub const PN532_SPI_DATAREAD: u8 = 0x03;
pub const PN532_SPI_READY: u8 = 0x01;

pub struct SPIInterface<SPI> {
    pub spi: SPI,
}

impl<SPI> Interface for SPIInterface<SPI>
where
    SPI: Transactional<u8>,
{
    type Error = SPI::Error;
    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.spi.exec(&mut [
            Operation::Write(&[PN532_SPI_DATAWRITE]),
            Operation::Write(frame),
        ])
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.spi.exec(&mut [
            Operation::Write(&[PN532_SPI_DATAREAD]),
            Operation::Transfer(buf),
        ])
    }
}
