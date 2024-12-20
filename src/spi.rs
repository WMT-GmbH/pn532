//! SPI interfaces
//!
//! # Note:
//! The SPI peripheral must be in [`MODE_0`](embedded_hal::spi::MODE_0)
//!
//! The SPI peripheral should be in **lsb mode**.
//! If your peripheral cannot be set to **lsb mode** you need to enable the `msb-spi` feature of this crate.
#[cfg(feature = "is_sync")]
use core::convert::Infallible;
use core::fmt::Debug;
#[cfg(feature = "is_sync")]
use core::task::Poll;

#[cfg(feature = "is_sync")]
use embedded_hal::digital::InputPin;

#[cfg(feature = "is_sync")]
use embedded_hal::spi::{Operation, SpiDevice};
#[cfg(not(feature = "is_sync"))]
use embedded_hal_async::spi::{Operation, SpiDevice};

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

#[maybe_async::maybe_async(AFIT)]
impl<SPI> Interface for SPIInterface<SPI>
where
    SPI: SpiDevice,
{
    type Error = <SPI as embedded_hal::spi::ErrorType>::Error;
    async fn wake_up(&mut self) -> Result<(), Self::Error> {
        self.spi
            .transaction(&mut [Operation::DelayNs(2_000_000)])
            .await
    }

    async fn write(&mut self, frame: &mut [u8]) -> Result<(), Self::Error> {
        #[cfg(feature = "msb-spi")]
        for byte in frame.iter_mut() {
            *byte = byte.reverse_bits();
        }
        self.spi
            .transaction(&mut [
                Operation::Write(&[PN532_SPI_DATAWRITE]),
                Operation::Write(frame),
            ])
            .await
    }

    #[maybe_async::sync_impl]
    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>> {
        let mut buf = [PN532_SPI_STATREAD, 0x00];

        self.spi.transfer_in_place(&mut buf)?;

        if buf[1] == PN532_SPI_READY {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    #[maybe_async::async_impl]
    async fn wait_ready(&mut self) -> Result<(), Self::Error> {
        let mut buf = [PN532_SPI_STATREAD, 0x00];

        while buf[1] != PN532_SPI_READY {
            buf = [PN532_SPI_STATREAD, 0x00];
            self.spi.transfer_in_place(&mut buf).await?;
        }
        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.spi
            .transaction(&mut [
                Operation::Write(&[PN532_SPI_DATAREAD]),
                Operation::Read(buf),
            ])
            .await?;

        #[cfg(feature = "msb-spi")]
        for byte in buf.iter_mut() {
            *byte = byte.reverse_bits();
        }
        Ok(())
    }
}

/// SPI Interface with IRQ pin
#[maybe_async::sync_impl]
#[derive(Clone, Debug)]
#[maybe_async::sync_impl]
pub struct SPIInterfaceWithIrq<SPI, IRQ>
where
    SPI: SpiDevice,
    IRQ: InputPin<Error = Infallible>,
{
    pub spi: SPI,
    pub irq: IRQ,
}

#[maybe_async::sync_impl]
impl<SPI, IRQ> Interface for SPIInterfaceWithIrq<SPI, IRQ>
where
    SPI: SpiDevice,
    IRQ: InputPin<Error = Infallible>,
{
    type Error = <SPI as embedded_hal::spi::ErrorType>::Error;

    fn wake_up(&mut self) -> Result<(), Self::Error> {
        self.spi.transaction(&mut [Operation::DelayNs(2_000_000)])
    }

    fn write(&mut self, frame: &mut [u8]) -> Result<(), Self::Error> {
        #[cfg(feature = "msb-spi")]
        for byte in frame.iter_mut() {
            *byte = byte.reverse_bits();
        }
        self.spi.transaction(&mut [
            Operation::Write(&[PN532_SPI_DATAWRITE]),
            Operation::Write(frame),
        ])
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
        self.spi.transaction(&mut [
            Operation::Write(&[PN532_SPI_DATAREAD]),
            Operation::Read(buf),
        ])?;

        #[cfg(feature = "msb-spi")]
        for byte in buf.iter_mut() {
            *byte = byte.reverse_bits();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i2c::tests::PinMock;
    use embedded_hal_mock::eh1::digital::State;
    use embedded_hal_mock::eh1::digital::Transaction as DigitalTransaction;
    use embedded_hal_mock::eh1::spi::Mock as SpiMock;
    use embedded_hal_mock::eh1::spi::Transaction as SpiTransaction;

    #[test]
    fn test_spi() {
        let mut spi = SPIInterface {
            spi: SpiMock::new(&[
                // write
                SpiTransaction::transaction_start(),
                SpiTransaction::write(as_lsb(0x01)),
                SpiTransaction::write_vec(vec![as_lsb(1), as_lsb(2)]),
                SpiTransaction::transaction_end(),
                // wait_ready
                SpiTransaction::transaction_start(),
                SpiTransaction::transfer_in_place(
                    vec![as_lsb(0x02), as_lsb(0x00)],
                    vec![as_lsb(0x00), as_lsb(0x00)],
                ),
                SpiTransaction::transaction_end(),
                SpiTransaction::transaction_start(),
                SpiTransaction::transfer_in_place(
                    vec![as_lsb(0x02), as_lsb(0x00)],
                    vec![as_lsb(0x00), as_lsb(0x01)],
                ),
                SpiTransaction::transaction_end(),
                // read
                SpiTransaction::transaction_start(),
                SpiTransaction::write(as_lsb(0x03)),
                SpiTransaction::read_vec(vec![as_lsb(3), as_lsb(4)]),
                SpiTransaction::transaction_end(),
            ]),
        };

        spi.write(&mut [1, 2]).unwrap();

        assert_eq!(spi.wait_ready(), Poll::Pending);
        assert_eq!(spi.wait_ready(), Poll::Ready(Ok(())));

        let mut buf = [0, 0];
        spi.read(&mut buf).unwrap();
        assert_eq!(buf, [3, 4]);

        spi.spi.done();
    }

    #[test]
    fn test_spi_with_irq() {
        let mut spi = SPIInterfaceWithIrq {
            spi: SpiMock::new(&[
                // write
                SpiTransaction::transaction_start(),
                SpiTransaction::write(as_lsb(0x01)),
                SpiTransaction::write_vec(vec![as_lsb(1), as_lsb(2)]),
                SpiTransaction::transaction_end(),
                // read
                SpiTransaction::transaction_start(),
                SpiTransaction::write(as_lsb(0x03)),
                SpiTransaction::read_vec(vec![as_lsb(3), as_lsb(4)]),
                SpiTransaction::transaction_end(),
            ]),
            irq: PinMock::new(&[
                DigitalTransaction::get(State::High),
                DigitalTransaction::get(State::Low),
            ]),
        };

        spi.write(&mut [1, 2]).unwrap();

        assert_eq!(spi.wait_ready(), Poll::Pending);
        assert_eq!(spi.wait_ready(), Poll::Ready(Ok(())));

        let mut buf = [0, 0];
        spi.read(&mut buf).unwrap();
        assert_eq!(buf, [3, 4]);

        spi.spi.done();
        spi.irq.mock.done();
    }
}
