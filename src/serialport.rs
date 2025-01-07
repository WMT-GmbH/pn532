//! SerialPort interface

use core::task::Poll;
use serialport::SerialPort;
use std::io::Write;

use crate::Interface;

/// SerialPort Interface without IRQ pin
pub struct SerialPortInterface {
    pub port: Box<dyn SerialPort>,
}

impl Interface for SerialPortInterface {
    type Error = std::io::Error;

    fn write(&mut self, frame: &mut [u8]) -> Result<(), Self::Error> {
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

impl SerialPortInterface {
    /// Wake the interface after a power down
    pub fn send_wakeup_message(&mut self) -> Result<(), std::io::Error> {
        // See "HSU wake up condition" on p.99 of the User Manual
        self.port.write_all(&[
            0x55, 0x55, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ])
    }
}
