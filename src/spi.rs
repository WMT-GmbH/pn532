/*use crate::Interface;
use core::convert::Infallible;
use core::fmt::Debug;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

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
    type WaitReadyFuture<'a>
    where
        SPI: 'a,
        CS: 'a,
    = &'a mut Self;

    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.cs.set_low().ok();
        self.spi.write(&[PN532_SPI_DATAWRITE])?;
        self.spi.write(frame)?;
        self.cs.set_high().ok();
        Ok(())
    }

    fn wait_ready(&mut self) -> Self::WaitReadyFuture<'_> {
        // with async traits this would be:
        // core::future::poll_fn(|_|{
        //     self.spi.write(&[PN532_SPI_STATREAD])?;
        //     let mut buf = [0x00];
        //     self.spi.transfer(&mut buf)?;
        //     if buf[0] == PN532_SPI_READY {
        //         Poll::Ready(Ok(()))
        //     } else {
        //         Poll::Pending
        //     }
        // })

        self
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.cs.set_low().ok();
        self.spi.write(&[PN532_SPI_DATAWRITE])?;
        self.spi.transfer(buf)?;
        self.cs.set_high().ok();
        Ok(())
    }
}

impl<'a, SPI, CS> Future for &'a mut SPIInterface<SPI, CS>
where
    SPI: Transfer<u8>,
    SPI: Write<u8, Error = <SPI as Transfer<u8>>::Error>,
    CS: OutputPin<Error = Infallible>,
{
    type Output = Result<(), <SPI as Transfer<u8>>::Error>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.cs.set_low().ok();
        self.spi.write(&[PN532_SPI_STATREAD])?;
        let mut buf = [0x00];
        self.spi.transfer(&mut buf)?;
        self.cs.set_high().ok();
        if buf[0] == PN532_SPI_READY {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
}
*/
