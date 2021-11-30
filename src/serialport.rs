//! SerialPort interface

use core::task::Poll;
use std::io::Write;
use std::time::{Duration, Instant};

use embedded_hal::timer::CountDown;
use serialport::SerialPort;

use crate::Interface;

/// SerialPort Interface without IRQ pin
pub struct SerialPortInterface {
    pub port: Box<dyn SerialPort>,
}

impl Interface for SerialPortInterface {
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

/// A timer based on [`std::time::Instant`], which is a monotonically nondecreasing clock.
pub struct SysTimer {
    start: Instant,
    duration: Duration,
}

impl SysTimer {
    pub fn new() -> SysTimer {
        SysTimer {
            start: Instant::now(),
            duration: Duration::from_millis(0),
        }
    }
}

impl Default for SysTimer {
    fn default() -> SysTimer {
        SysTimer::new()
    }
}

impl CountDown for SysTimer {
    type Time = Duration;

    fn start<T>(&mut self, count: T)
    where
        T: Into<Self::Time>,
    {
        self.start = Instant::now();
        self.duration = count.into();
    }

    fn wait(&mut self) -> nb::Result<(), void::Void> {
        if (Instant::now() - self.start) >= self.duration {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}
