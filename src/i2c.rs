//! I2C interfaces
use core::fmt::Debug;

#[cfg(feature = "is_sync")]
use core::convert::Infallible;
#[cfg(feature = "is_sync")]
use core::task::Poll;
#[cfg(feature = "is_sync")]
use embedded_hal::digital::InputPin;
#[cfg(feature = "is_sync")]
use embedded_hal::i2c::I2c as embedded_I2c;

#[cfg(not(feature = "is_sync"))]
use embedded_hal_async::i2c::I2c as embedded_I2c;

use crate::Interface;
use embedded_hal::i2c::Operation;

/// To be used in `Interface::wait_ready` implementations
pub const PN532_I2C_READY: u8 = 0x01;

/// I2C address of the Pn532
pub const I2C_ADDRESS: u8 = 0x24;

/// I2C Interface without IRQ pin
///
/// # Note:
/// Currently the implementation of [`I2CInterface::wait_ready`] ignores any I2C errors.
/// See this [issue](https://github.com/WMT-GmbH/pn532/issues/4) for an explanation.
#[derive(Clone, Debug)]
pub struct I2CInterface<I2C>
where
    I2C: embedded_I2c,
{
    pub i2c: I2C,
}

#[maybe_async::maybe_async(AFIT)]
impl<I2C> Interface for I2CInterface<I2C>
where
    I2C: embedded_I2c,
{
    type Error = I2C::Error;

    async fn write(&mut self, frame: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.write(I2C_ADDRESS, frame).await
    }

    // wait_ready implementations differ between sync / async

    #[maybe_async::sync_impl]
    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>> {
        let mut buf = [0];
        self.i2c.read(I2C_ADDRESS, &mut buf).ok();
        // It's possible that the PN532 does not ACK the read request when it is not ready.
        // Since we don't know the concrete type of `Self::Error` unfortunately we have to ignore all interface errors here.
        // See https://github.com/WMT-GmbH/pn532/issues/4 for more info

        if buf[0] == PN532_I2C_READY {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
    #[maybe_async::async_impl]
    async fn wait_ready(&mut self) -> Result<(), Self::Error> {
        let mut buf = [0];
        while buf[0] != PN532_I2C_READY {
            let _ = self.i2c.read(I2C_ADDRESS, &mut buf).await;
            // It's possible that the PN532 does not ACK the read request when it is not ready.
            // Since we don't know the concrete type of `Self::Error` unfortunately we have to ignore all interface errors here.
            // See https://github.com/WMT-GmbH/pn532/issues/4 for more info
        }
        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c
            .transaction(
                I2C_ADDRESS,
                &mut [Operation::Read(&mut [0]), Operation::Read(buf)],
            )
            .await
    }
}

/// I2C Interface with IRQ pin
#[maybe_async::sync_impl]
#[derive(Clone, Debug)]
#[maybe_async::sync_impl]
pub struct I2CInterfaceWithIrq<I2C, IRQ>
where
    I2C: embedded_hal::i2c::I2c,
    IRQ: InputPin<Error = Infallible>,
{
    pub i2c: I2C,
    pub irq: IRQ,
}

#[maybe_async::sync_impl]
impl<I2C, IRQ> Interface for I2CInterfaceWithIrq<I2C, IRQ>
where
    I2C: embedded_hal::i2c::I2c,
    IRQ: InputPin<Error = Infallible>,
{
    type Error = <I2C as embedded_hal::i2c::ErrorType>::Error;

    fn write(&mut self, frame: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.write(I2C_ADDRESS, frame)
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
        self.i2c.transaction(
            I2C_ADDRESS,
            &mut [Operation::Read(&mut [0]), Operation::Read(buf)],
        )
    }
}
