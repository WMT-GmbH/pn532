#![feature(future_poll_fn)]
#![feature(const_generics_defaults)]
#![no_std]

use core::fmt::Debug;
use core::task::Poll;

pub use crate::protocol::{Error, Pn532};
pub use crate::requests::Request;

pub mod i2c;
mod protocol;
pub mod requests;
pub mod spi;

pub trait Interface {
    type Error: Debug;
    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error>;
    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>>;
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error>;
}

impl<I: Interface> Interface for &mut I {
    type Error = I::Error;

    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        I::write(self, frame)
    }

    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>> {
        I::wait_ready(self)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        I::read(self, buf)
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ErrorStatus {
    Timeout = 0x01,
    CrcError = 0x02,
    ParityError = 0x03,
    WrongBitCountDuringAntiCollision = 0x04,
    FramingError = 0x05,
    AbnormalBitCollision = 0x06,
    InsufficientCommunicationBuffer = 0x07,
    RfBufferOverflow = 0x09,
    RfFieldHasNotBeenSwitchedOn = 0x0A,
    RfProtocolError = 0x0B,
    Overheating = 0x0D,
    InternalBufferOverflow = 0x0E,
    InvalidParameter = 0x10,
    CommandNotSupported = 0x12,
    WrongDataFormat = 0x13,
    AuthenticationError = 0x14,
    WrongUidCheckByte = 0x23,
    InvalidDeviceState = 0x25,
    OperationNotAllowed = 0x26,
    CommandNotAcceptable = 0x27,
    TargetHasBeenReleased = 0x29,
    CardHasBeenExchanged = 0x2A,
    CardHasDisappeared = 0x2B,
    Nfcid3InitiatorTargetMismatch = 0x2C,
    OverCurrent = 0x2D,
    NadMsssing = 0x2E,
}

impl core::convert::TryFrom<u8> for ErrorStatus {
    type Error = ();

    fn try_from(value: u8) -> Result<ErrorStatus, ()> {
        let value = value & 0b0011_1111;
        match value {
            0x01 => Ok(ErrorStatus::Timeout),
            0x02 => Ok(ErrorStatus::CrcError),
            0x03 => Ok(ErrorStatus::ParityError),
            0x04 => Ok(ErrorStatus::WrongBitCountDuringAntiCollision),
            0x05 => Ok(ErrorStatus::FramingError),
            0x06 => Ok(ErrorStatus::AbnormalBitCollision),
            0x07 => Ok(ErrorStatus::InsufficientCommunicationBuffer),
            0x09 => Ok(ErrorStatus::RfBufferOverflow),
            0x0A => Ok(ErrorStatus::RfFieldHasNotBeenSwitchedOn),
            0x0B => Ok(ErrorStatus::RfProtocolError),
            0x0D => Ok(ErrorStatus::Overheating),
            0x0E => Ok(ErrorStatus::InternalBufferOverflow),
            0x10 => Ok(ErrorStatus::InvalidParameter),
            0x12 => Ok(ErrorStatus::CommandNotSupported),
            0x13 => Ok(ErrorStatus::WrongDataFormat),
            0x14 => Ok(ErrorStatus::AuthenticationError),
            0x23 => Ok(ErrorStatus::WrongUidCheckByte),
            0x25 => Ok(ErrorStatus::InvalidDeviceState),
            0x26 => Ok(ErrorStatus::OperationNotAllowed),
            0x27 => Ok(ErrorStatus::CommandNotAcceptable),
            0x29 => Ok(ErrorStatus::TargetHasBeenReleased),
            0x2A => Ok(ErrorStatus::CardHasBeenExchanged),
            0x2B => Ok(ErrorStatus::CardHasDisappeared),
            0x2C => Ok(ErrorStatus::Nfcid3InitiatorTargetMismatch),
            0x2D => Ok(ErrorStatus::OverCurrent),
            0x2E => Ok(ErrorStatus::NadMsssing),
            _ => Err(()),
        }
    }
}
