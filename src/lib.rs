#![feature(future_poll_fn)]
#![no_std]

use core::fmt::Debug;
use core::task::Poll;

pub use crate::protocol::{Error, Pn532};

mod protocol;
pub mod spi;
pub mod tag;

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
pub enum Command {
    Diagnose = 0x00,
    GetFirmwareVersion = 0x02,
    GetGeneralStatus = 0x04,
    ReadRegister = 0x06,
    WriteRegister = 0x08,
    ReadGPIO = 0x0C,
    WriteGPIO = 0x0E,
    SetSerialBaudRate = 0x10,
    SetParameters = 0x12,
    SAMConfiguration = 0x14,
    PowerDown = 0x16,
    RFConfiguration = 0x32,
    RFRegulationTest = 0x58,
    InJumpForDEP = 0x56,
    InJumpForPSL = 0x46,
    InListPassiveTarget = 0x4A,
    InATR = 0x50,
    InPSL = 0x4E,
    InDataExchange = 0x40,
    InCommunicateThru = 0x42,
    InDeselect = 0x44,
    InRelease = 0x52,
    InSelect = 0x54,
    InAutoPoll = 0x60,
    TgInitAsTarget = 0x8C,
    TgSetGeneralBytes = 0x92,
    TgGetData = 0x86,
    TgSetData = 0x8E,
    TgSetMetaData = 0x94,
    TgGetInitiatorCommand = 0x88,
    TgResponseToInitiator = 0x90,
    TgGetTargetStatus = 0x8A,
}

pub enum SAMMode {
    /// The SAM is not used; this is the default mode
    Normal,
    /// The couple PN532+SAM is seen as only one contactless SAM card
    /// from the external world
    VirtualCard {
        /// In multiples of 50ms
        timeout: u8,
    },
    /// The host controller can access to the SAM with standard PCD commands
    /// (InListPassiveTarget, InDataExchange, ...)
    WiredCard,
    /// Both the PN532 and the SAM are visible from the external world
    /// as two separated targets
    DualCard,
}

#[repr(u8)]
pub enum CardType {
    /// 106 kbps type A (ISO/IEC14443 Type A)
    IsoTypeA = 0x00,
    /// 212 kbps (FeliCa polling)
    FeliCa212kbps = 0x01,
    /// 424 kbps (FeliCa polling)
    FeliCa424kbps = 0x02,
    /// 106 kbps type B (ISO/IEC14443-3B)
    IsoTypeB = 0x03,
    /// 106 kbps Innovision Jewel tag
    Jewel = 0x04,
}

#[repr(u8)]
pub enum TxSpeed {
    Tx106kbps = 0b0000_0000,
    Tx212kbps = 0b0001_0000,
    Tx424kbps = 0b0010_0000,
    Tx848kbps = 0b0011_0000,
}

#[repr(u8)]
pub enum TxFraming {
    Mifare = 0b0000_0000,
    FeliCa = 0b0000_0010,
}

impl Pn532<(), ()> {
    pub const GET_FIRMWARE_VERSION: [u8; 1 + 8] =
        Pn532::make_frame(&[Command::GetFirmwareVersion as u8]);
    pub const INLIST_ONE_ISO_A_TARGET: [u8; 3 + 8] = Pn532::make_frame(&[
        Command::InListPassiveTarget as u8,
        1,
        CardType::IsoTypeA as u8,
    ]);
    /// Make a SAMConfiguration frame
    pub const fn sam_configuration_frame(mode: SAMMode, use_irq_pin: bool) -> [u8; 4 + 8] {
        let (mode, timeout) = match mode {
            SAMMode::Normal => (1, 0),
            SAMMode::VirtualCard { timeout } => (2, timeout),
            SAMMode::WiredCard => (3, 0),
            SAMMode::DualCard => (4, 0),
        };
        Pn532::make_frame(&[
            Command::SAMConfiguration as u8,
            mode,
            timeout,
            !use_irq_pin as u8,
        ])
    }

    pub const fn rf_regulation_test_frame(tx_speed: TxSpeed, tx_framing: TxFraming) -> [u8; 2 + 8] {
        Pn532::make_frame(&[
            Command::RFRegulationTest as u8,
            tx_speed as u8 | tx_framing as u8,
        ])
    }

    pub const SELECT_TAG_1: [u8; 2 + 8] = Pn532::make_frame(&[Command::InSelect as u8, 1]);
    pub const SELECT_TAG_2: [u8; 2 + 8] = Pn532::make_frame(&[Command::InSelect as u8, 2]);
    pub const DESELECT_TAG_1: [u8; 2 + 8] = Pn532::make_frame(&[Command::InDeselect as u8, 1]);
    pub const DESELECT_TAG_2: [u8; 2 + 8] = Pn532::make_frame(&[Command::InDeselect as u8, 2]);
    pub const RELEASE_TAG_1: [u8; 2 + 8] = Pn532::make_frame(&[Command::InRelease as u8, 1]);
    pub const RELEASE_TAG_2: [u8; 2 + 8] = Pn532::make_frame(&[Command::InRelease as u8, 2]);

    // TODO power down
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
