//! `no_std` implementation of the [`Pn532`] protocol using `embedded_hal` traits.
//!
//! Since communication with the Pn532 can be rather slow at times,
//! communication can be split into multiple parts, a timeout can be provided or an async runtime
//! can be used.
//!
//! The Pn532 supports different serial links. The [`Interface`] trait abstracts
//! over these different links.
//!
//! `Interface` can be manually implemented or one these provided interface structs can be used:
//! * [`spi::SPIInterface`]
//! * [`spi::SPIInterfaceWithIrq`]
//! * [`i2c::I2CInterface`]
//! * [`i2c::I2CInterfaceWithIrq`]
//! * [`serialport::SerialPortInterface`]
//!
//! # SPI example
//! ```
//! # use pn532::doc_test_helper::{NoOpSPI, NoOpCS, NoOpTimer};
//! use pn532::{requests::SAMMode, spi::SPIInterface, Pn532, Request};
//! use pn532::IntoDuration; // trait for `ms()`, your HAL might have its own
//!
//! # let spi = NoOpSPI;
//! # let cs = NoOpCS;
//! # let timer = NoOpTimer;
//! #
//! // spi, cs and timer are structs implementing their respective embedded_hal traits.
//!
//! let interface = SPIInterface {
//!     spi,
//!     cs,
//! };
//! let mut pn532: Pn532<_, _, 32> = Pn532::new(interface, timer);
//! if let Err(e) = pn532.process(&Request::sam_configuration(SAMMode::Normal, false), 0, 50.ms()){
//!     println!("Could not initialize PN532: {e:?}")
//! }
//! if let Ok(uid) = pn532.process(&Request::INLIST_ONE_ISO_A_TARGET, 7, 1000.ms()){
//!     let result = pn532.process(&Request::ntag_read(10), 17, 50.ms()).unwrap();
//!     if result[0] == 0x00 {
//!         println!("page 10: {:?}", &result[1..5]);
//!     }
//! }
//! ```
//!
//! # `msb-spi` feature
//! If you want to use either [`spi::SPIInterface`] or [`spi::SPIInterfaceWithIrq`] and
//! your peripheral cannot be set to **lsb mode** you need to enable the `msb-spi` feature of this crate.
//!
//! # `std` feature
//! Enable the std feature to use [`serialport::SerialPortInterface`]
//! Only works for [targets](https://github.com/serialport/serialport-rs#platform-support) supported by the `serialport` crate.

#![cfg_attr(not(any(feature = "std", doc)), no_std)]
#![cfg_attr(doc, feature(doc_cfg))]

use core::fmt::Debug;
use core::task::Poll;
use core::time::Duration;

pub use crate::protocol::{Error, Pn532};
pub use crate::requests::Request;

pub mod i2c;
mod protocol;
pub mod requests;
#[cfg(feature = "std")]
#[cfg_attr(doc, doc(cfg(feature = "std")))]
pub mod serialport;
pub mod spi;

/// Abstraction over the different serial links.
/// Either SPI, I2C or HSU (High Speed UART).
pub trait Interface {
    /// Error specific to the serial link.
    type Error: Debug;
    /// Writes a `frame` to the Pn532
    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error>;
    /// Checks if the Pn532 has data to be read.
    /// Uses either the serial link or the IRQ pin.
    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>>;
    /// Reads data from the Pn532 into `buf`.
    /// This method will only be called if `wait_ready` returned `Poll::Ready(Ok(()))` before.
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

/// Some commands return a status byte.
/// If this byte is not zero it will contain an `ErrorCode`.
///
/// ```
/// # use pn532::ErrorCode;
/// fn print_error(status_byte: u8){
///     if let Ok(error_code) = ErrorCode::try_from(status_byte){
///         println!("{:?}", error_code);
///     } else {
///         println!("unknown error code");
///     }
/// }
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ErrorCode {
    /// Time Out, the target has not answered
    Timeout = 0x01,
    /// A CRC error has been detected by the CIU
    CrcError = 0x02,
    /// A Parity error has been detected by the CIU
    ParityError = 0x03,
    /// During an anti-collision/select operation (ISO/IEC14443-3
    /// Type A and ISO/IEC18092 106 kbps passive mode), an
    /// erroneous Bit Count has been detected
    WrongBitCountDuringAntiCollision = 0x04,
    /// Framing error during Mifare operation
    FramingError = 0x05,
    /// An abnormal bit-collision has been detected during bit wise
    /// anti-collision at 106 kbps
    AbnormalBitCollision = 0x06,
    /// Communication buffer size insufficient
    InsufficientCommunicationBuffer = 0x07,
    /// RF Buffer overflow has been detected by the CIU (bit
    /// BufferOvfl of the register CIU_Error)
    RfBufferOverflow = 0x09,
    /// In active communication mode, the RF field has not been
    /// switched on in time by the counterpart (as defined in NFCIP-1
    /// standard)
    RfFieldHasNotBeenSwitchedOn = 0x0A,
    /// RF Protocol error (cf. Error! Reference source not found.,
    /// description of the CIU_Error register)
    RfProtocolError = 0x0B,
    /// Temperature error: the internal temperature sensor has
    /// detected overheating, and therefore has automatically
    /// switched off the antenna drivers
    Overheating = 0x0D,
    /// Internal buffer overflow
    InternalBufferOverflow = 0x0E,
    /// Invalid parameter (range, format, …)
    InvalidParameter = 0x10,
    /// DEP Protocol: The PN532 configured in target mode does not
    /// support the command received from the initiator (the
    /// command received is not one of the following: ATR_REQ,
    /// WUP_REQ, PSL_REQ, DEP_REQ, DSL_REQ, RLS_REQ
    /// Error! Reference source not found.).
    CommandNotSupported = 0x12,
    /// DEP Protocol, Mifare or ISO/IEC14443-4: The data format
    /// does not match to the specification.
    /// Depending on the RF protocol used, it can be:
    /// • Bad length of RF received frame,
    /// • Incorrect value of PCB or PFB,
    /// • Invalid or unexpected RF received frame,
    /// • NAD or DID incoherence.
    WrongDataFormat = 0x13,
    /// Mifare: Authentication error
    AuthenticationError = 0x14,
    /// ISO/IEC14443-3: UID Check byte is wrong
    WrongUidCheckByte = 0x23,
    /// DEP Protocol: Invalid device state, the system is in a state
    /// which does not allow the operation
    InvalidDeviceState = 0x25,
    /// Operation not allowed in this configuration (host controller
    /// interface)
    OperationNotAllowed = 0x26,
    /// This command is not acceptable due to the current context of
    /// the PN532 (Initiator vs. Target, unknown target number,
    /// Target not in the good state, …)
    CommandNotAcceptable = 0x27,
    /// The PN532 configured as target has been released by its
    /// initiator
    TargetHasBeenReleased = 0x29,
    /// PN532 and ISO/IEC14443-3B only: the ID of the card does
    /// not match, meaning that the expected card has been
    /// exchanged with another one.
    CardHasBeenExchanged = 0x2A,
    /// PN532 and ISO/IEC14443-3B only: the card previously
    /// activated has disappeared.
    CardHasDisappeared = 0x2B,
    /// Mismatch between the NFCID3 initiator and the NFCID3
    /// target in DEP 212/424 kbps passive.
    NfcId3InitiatorTargetMismatch = 0x2C,
    /// An over-current event has been detected
    OverCurrent = 0x2D,
    /// NAD missing in DEP frame
    NadMissing = 0x2E,
}

impl TryFrom<u8> for ErrorCode {
    type Error = ();

    fn try_from(value: u8) -> Result<ErrorCode, ()> {
        let value = value & 0b0011_1111;
        match value {
            0x01 => Ok(ErrorCode::Timeout),
            0x02 => Ok(ErrorCode::CrcError),
            0x03 => Ok(ErrorCode::ParityError),
            0x04 => Ok(ErrorCode::WrongBitCountDuringAntiCollision),
            0x05 => Ok(ErrorCode::FramingError),
            0x06 => Ok(ErrorCode::AbnormalBitCollision),
            0x07 => Ok(ErrorCode::InsufficientCommunicationBuffer),
            0x09 => Ok(ErrorCode::RfBufferOverflow),
            0x0A => Ok(ErrorCode::RfFieldHasNotBeenSwitchedOn),
            0x0B => Ok(ErrorCode::RfProtocolError),
            0x0D => Ok(ErrorCode::Overheating),
            0x0E => Ok(ErrorCode::InternalBufferOverflow),
            0x10 => Ok(ErrorCode::InvalidParameter),
            0x12 => Ok(ErrorCode::CommandNotSupported),
            0x13 => Ok(ErrorCode::WrongDataFormat),
            0x14 => Ok(ErrorCode::AuthenticationError),
            0x23 => Ok(ErrorCode::WrongUidCheckByte),
            0x25 => Ok(ErrorCode::InvalidDeviceState),
            0x26 => Ok(ErrorCode::OperationNotAllowed),
            0x27 => Ok(ErrorCode::CommandNotAcceptable),
            0x29 => Ok(ErrorCode::TargetHasBeenReleased),
            0x2A => Ok(ErrorCode::CardHasBeenExchanged),
            0x2B => Ok(ErrorCode::CardHasDisappeared),
            0x2C => Ok(ErrorCode::NfcId3InitiatorTargetMismatch),
            0x2D => Ok(ErrorCode::OverCurrent),
            0x2E => Ok(ErrorCode::NadMissing),
            _ => Err(()),
        }
    }
}

/// Extension trait with convenience methods for turning `u64` into `Duration`
pub trait IntoDuration {
    fn ms(self) -> Duration;
    fn us(self) -> Duration;
}

impl IntoDuration for u64 {
    fn ms(self) -> Duration {
        Duration::from_millis(self)
    }
    fn us(self) -> Duration {
        Duration::from_micros(self)
    }
}

#[doc(hidden)]
// FIXME: #[cfg(doctest)] once https://github.com/rust-lang/rust/issues/67295 is fixed.
pub mod doc_test_helper;
