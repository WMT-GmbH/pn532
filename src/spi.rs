//! SPI interfaces
//!
//! # Note:
//! The SPI peripheral must be in [`MODE_0`](embedded_hal::spi::MODE_0)
//!
//! The SPI peripheral should be in **lsb mode**.
//! If your peripheral cannot be set to **lsb mode** you need to enable the `msb-spi` feature of this crate.
use core::convert::Infallible;
use core::fmt::Debug;
use core::task::Poll;

use embedded_hal::spi::SpiDevice;
use embedded_hal::digital::{InputPin, OutputPin};

use crate::Interface;

#[cfg(feature = "msb-spi")]
const fn as_lsb(byte: u8) -> u8 {
    byte.reverse_bits()
}
#[cfg(not(feature = "msb-spi"))]
const fn as_lsb(byte: u8) -> u8 {
    byte
}

/// To be used in `Interface::wait_ready` implementations
pub const PN532_SPI_STATREAD: u8 = as_lsb(0x02);
/// To be used in `Interface::write` implementations
pub const PN532_SPI_DATAWRITE: u8 = as_lsb(0x01);
/// To be used in `Interface::read` implementations
pub const PN532_SPI_DATAREAD: u8 = as_lsb(0x03);
/// To be used in `Interface::wait_ready` implementations
pub const PN532_SPI_READY: u8 = as_lsb(0x01);

/// SPI Interface without IRQ pin
#[derive(Clone, Debug)]
pub struct SPIInterface<SPI>
where
    SPI: SpiDevice,
{
    pub spi: SPI,
}

impl<SPI> Interface for SPIInterface<SPI>
where
    SPI: SpiDevice,
{
    type Error = <SPI as embedded_hal::spi::ErrorType>::Error;

    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.spi.write(&[PN532_SPI_DATAWRITE])?;

        #[cfg(feature = "msb-spi")]
        for byte in frame {
            self.spi.write(&[byte.reverse_bits()])?
        }
        #[cfg(not(feature = "msb-spi"))]
        self.spi.write(frame)?;

        Ok(())
    }

    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>> {
        let mut buf = [0x00];

        self.spi.write(&[PN532_SPI_STATREAD])?;
        self.spi.transfer_in_place(&mut buf)?;

        if buf[0] == PN532_SPI_READY {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.spi.write(&[PN532_SPI_DATAREAD])?;
        self.spi.transfer_in_place(buf)?;

        #[cfg(feature = "msb-spi")]
        for byte in buf.iter_mut() {
            *byte = byte.reverse_bits();
        }
        Ok(())
    }
}

/// SPI Interface with IRQ pin
#[derive(Clone, Debug)]
pub struct SPIInterfaceWithIrq<SPI, IRQ>
where
    SPI: SpiDevice,
    IRQ: InputPin<Error = Infallible>,
{
    pub spi: SPI,
    pub irq: IRQ,
}

impl<SPI, IRQ> Interface for SPIInterfaceWithIrq<SPI, IRQ>
where
    SPI: SpiDevice,
    IRQ: InputPin<Error = Infallible>,
{
    type Error = <SPI as embedded_hal::spi::ErrorType>::Error;

    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.spi.write(&[PN532_SPI_DATAWRITE])?;

        #[cfg(feature = "msb-spi")]
        for byte in frame {
            self.spi.write(&[byte.reverse_bits()])?
        }
        #[cfg(not(feature = "msb-spi"))]
        self.spi.write(frame)?;

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
        self.spi.write(&[PN532_SPI_DATAREAD])?;
        self.spi.transfer_in_place(buf)?;

        #[cfg(feature = "msb-spi")]
        for byte in buf.iter_mut() {
            *byte = byte.reverse_bits();
        }
        Ok(())
    }
}
