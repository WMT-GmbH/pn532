use core::convert::Infallible;
use core::fmt::Debug;
use core::task::Poll;

use embedded_hal::blocking::i2c::{Operation, Read, Transactional, Write};
use embedded_hal::digital::v2::InputPin;

use crate::Interface;

const PN532_I2C_READY: u8 = 0x01;

const I2C_ADDRESS: u8 = 0x24;

pub struct I2CInterface<I2C> {
    pub i2c: I2C,
}

impl<I2C> Interface for I2CInterface<I2C>
where
    I2C: Transactional,
    I2C: Write<Error = <I2C as Transactional>::Error>,
    I2C: Read<Error = <I2C as Transactional>::Error>,
    <I2C as Transactional>::Error: Debug,
{
    type Error = <I2C as Transactional>::Error;

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
        self.i2c.exec(
            I2C_ADDRESS,
            &mut [Operation::Read(&mut [0]), Operation::Read(buf)],
        )
    }
}

pub struct I2CInterfaceWithIrq<I2C, IRQ> {
    pub i2c: I2C,
    pub irq: IRQ,
}

impl<I2C, IRQ> Interface for I2CInterfaceWithIrq<I2C, IRQ>
where
    I2C: Transactional,
    I2C: Write<Error = <I2C as Transactional>::Error>,
    I2C: Read<Error = <I2C as Transactional>::Error>,
    <I2C as Transactional>::Error: Debug,
    IRQ: InputPin<Error = Infallible>,
{
    type Error = <I2C as Transactional>::Error;

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
        self.i2c.exec(
            I2C_ADDRESS,
            &mut [Operation::Read(&mut [0]), Operation::Read(buf)],
        )
    }
}
