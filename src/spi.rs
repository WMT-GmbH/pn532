use core::convert::Infallible;
use core::fmt::Debug;
use core::task::Poll;

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::{InputPin, OutputPin};

use crate::Interface;

pub const PN532_SPI_STATREAD: u8 = 0x02;
pub const PN532_SPI_DATAWRITE: u8 = 0x01;
pub const PN532_SPI_DATAREAD: u8 = 0x03;
pub const PN532_SPI_READY: u8 = 0x01;

pub struct SPIInterface<SPI, CS> {
    pub spi: SPI,
    pub cs: CS,
}

impl<SPI, CS> Interface for SPIInterface<SPI, CS>
where
    SPI: Transfer<u8>,
    SPI: Write<u8, Error = <SPI as Transfer<u8>>::Error>,
    <SPI as Transfer<u8>>::Error: Debug,
    CS: OutputPin<Error = Infallible>,
{
    type Error = <SPI as Transfer<u8>>::Error;

    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.cs.set_low().ok();
        self.spi.write(&[PN532_SPI_DATAWRITE])?;
        self.spi.write(frame)?;
        self.cs.set_high().ok();
        Ok(())
    }

    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>> {
        self.spi.write(&[PN532_SPI_STATREAD])?;
        let mut buf = [0x00];
        self.spi.transfer(&mut buf)?;
        if buf[0] == PN532_SPI_READY {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.cs.set_low().ok();
        self.spi.write(&[PN532_SPI_DATAWRITE])?;
        self.spi.transfer(buf)?;
        self.cs.set_high().ok();
        Ok(())
    }
}

pub struct SPIInterfaceWithIrq<SPI, CS, IRQ> {
    pub spi: SPI,
    pub cs: CS,
    pub irq: IRQ,
}

impl<SPI, CS, IRQ> Interface for SPIInterfaceWithIrq<SPI, CS, IRQ>
where
    SPI: Transfer<u8>,
    SPI: Write<u8, Error = <SPI as Transfer<u8>>::Error>,
    <SPI as Transfer<u8>>::Error: Debug,
    CS: OutputPin<Error = Infallible>,
    IRQ: InputPin<Error = Infallible>,
{
    type Error = <SPI as Transfer<u8>>::Error;

    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.cs.set_low().ok();
        self.spi.write(&[PN532_SPI_DATAWRITE])?;
        self.spi.write(frame)?;
        self.cs.set_high().ok();
        Ok(())
    }

    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>> {
        // infallible unwrap because of IRQ bound
        if self.irq.is_low().unwrap() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.cs.set_low().ok();
        self.spi.write(&[PN532_SPI_DATAWRITE])?;
        self.spi.transfer(buf)?;
        self.cs.set_high().ok();
        Ok(())
    }
}
