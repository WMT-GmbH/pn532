//! SerialPort interfaces

use core::convert::Infallible;
use core::task::Poll;

use embedded_hal::digital::v2::InputPin;
use serialport::SerialPort;

use crate::Interface;

/// SerialPort Interface without IRQ pin
pub struct SerialPortInterface {
    pub port: Box<dyn SerialPort>,
}

impl Interface for SerialPortInterface
{
    type Error = std::io::Error;

    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.port.write_all(frame)
    }

    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>> {
        if self.port.bytes_to_read()? > 0 {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.port.read_exact(buf)
    }
}

/// SerialPort Interface with IRQ pin
pub struct SerialPortInterfaceWithIrq<IRQ>
where
    IRQ: InputPin<Error = Infallible>,
{
    pub port: Box<dyn SerialPort>,
    pub irq: IRQ,
}

impl<IRQ> Interface for SerialPortInterfaceWithIrq<IRQ>
where
    IRQ: InputPin<Error = Infallible>,
{
    type Error = std::io::Error;

    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.port.write_all(frame)
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
        self.port.read_exact(buf)
    }
}
