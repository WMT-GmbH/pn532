//! I2C interfaces
use core::convert::Infallible;
use core::fmt::Debug;
use core::task::Poll;

use embedded_hal::digital::InputPin;

use crate::Interface;

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
    I2C: embedded_hal::i2c::I2c,
{
    pub i2c: I2C,
}

impl<I2C> Interface for I2CInterface<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    type Error = I2C::Error;

    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.i2c.write(I2C_ADDRESS, frame)
    }

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

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let mut local_buf = [0u8;32];
        let local_buf_slice = &mut local_buf[..buf.len()+1]; // read one more than buf
        self.i2c.read( I2C_ADDRESS, local_buf_slice)?;
        buf.copy_from_slice(&local_buf_slice[1..]);
        Ok(())
    }
}

/// I2C Interface with IRQ pin
#[derive(Clone, Debug)]
pub struct I2CInterfaceWithIrq<I2C, IRQ>
where
    I2C: embedded_hal::i2c::I2c,
    IRQ: InputPin<Error = Infallible>,
{
    pub i2c: I2C,
    pub irq: IRQ,
}

impl<I2C, IRQ> Interface for I2CInterfaceWithIrq<I2C, IRQ>
where
    I2C: embedded_hal::i2c::I2c,
    IRQ: InputPin<Error = Infallible>,
{
    type Error = <I2C as embedded_hal::i2c::ErrorType>::Error;

    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
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
        self.i2c.read( I2C_ADDRESS,buf)
    }
}
