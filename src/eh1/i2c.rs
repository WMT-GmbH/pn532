//! I2C interfaces

use core::convert::Infallible;
use core::fmt::Debug;
use core::task::Poll;

use embedded_hal_1::i2c::{I2c, Operation};

use crate::Interface;

/// To be used in `Interface::wait_ready` implementations
pub const PN532_I2C_READY: u8 = 0x01;

/// I2C address of the Pn532
pub const I2C_ADDRESS: u8 = 0x24;

/// I2C Interface without IRQ pin
#[cfg(feature = "eh1")]
#[derive(Clone, Debug)]
pub struct I2CInterface<I2C>
where
    I2C: I2c,
    I2C::Error: Debug,
{
    pub i2c: I2C,
}

#[cfg(feature = "eh1")]
impl<I2C> Interface for I2CInterface<I2C>
where
    I2C: I2c,
    I2C::Error: Debug,
{
    type Error = I2C::Error;

    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.i2c.write(I2C_ADDRESS, frame)
    }

    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>> {
        let mut buf = [0];
        self.i2c.read(I2C_ADDRESS, &mut buf)?;

        if buf[0] == PN532_I2C_READY {
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

/// I2C Interface with IRQ pin
#[cfg(feature = "eh1")]
#[derive(Clone, Debug)]
pub struct I2CInterfaceWithIrq<I2C, IRQ>
where
    I2C: I2c,
    I2C::Error: Debug,
    IRQ: embedded_hal_1::digital::InputPin<Error = Infallible>,
{
    pub i2c: I2C,
    pub irq: IRQ,
}

#[cfg(feature = "eh1")]
impl<I2C, IRQ> Interface for I2CInterfaceWithIrq<I2C, IRQ>
where
    I2C: I2c,
    I2C::Error: Debug,
    IRQ: embedded_hal_1::digital::InputPin<Error = Infallible>,
{
    type Error = I2C::Error;

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
        self.i2c.transaction(
            I2C_ADDRESS,
            &mut [Operation::Read(&mut [0]), Operation::Read(buf)],
        )
    }
}
