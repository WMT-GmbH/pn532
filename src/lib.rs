#![feature(const_panic)]
#![no_std]

use core::fmt::Debug;

pub use crate::protocol::{make_frame, send_frame, Error};

mod protocol;
pub mod spi;

pub trait Interface {
    type Error: Debug;
    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error>;
    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error>;
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

/// Make a GetFirmwareVersion frame.
pub const fn get_firmware_version_frame() -> [u8; 9] {
    make_frame(&[Command::GetFirmwareVersion as u8])
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

/// Make a SAMConfiguration frame.
pub const fn sam_configuration_frame(mode: SAMMode, use_irq_pin: bool) -> [u8; 12] {
    let (mode, timeout) = match mode {
        SAMMode::Normal => (1, 0),
        SAMMode::VirtualCard { timeout } => (2, timeout),
        SAMMode::WiredCard => (3, 0),
        SAMMode::DualCard => (4, 0),
    };
    make_frame(&[
        Command::SAMConfiguration as u8,
        mode,
        timeout,
        !use_irq_pin as u8,
    ])
}

pub enum CardType {
    /// 106 kbps type A (ISO/IEC14443 Type A)
    IsoTypeA { max_tag_number: u8 },
    /// 106 kbps type B (ISO/IEC14443-3B)
    IsoTypeB { max_tag_number: u8 },
    /// 106 kbps Innovision Jewel tag
    Jewel,
}

/// Make a InListPassiveTarget frame.
///
/// The InListPassiveTarget also accepts FeliCa cards
/// and optional (in the case of FeliCa mandatory) "InitiatorData".
/// Use [`make_frame`] if you need these instead.
pub const fn inlist_passive_target_frame(card_type: CardType) -> [u8; 11] {
    let (max_tag_number, baud_rate) = match card_type {
        CardType::IsoTypeA { max_tag_number } => (max_tag_number, 0x00),
        CardType::IsoTypeB { max_tag_number } => (max_tag_number, 0x03),
        CardType::Jewel => (1, 0x00),
    };
    if max_tag_number != 1 || max_tag_number != 2 {
        panic!("max_tag_number must be 1 or 2")
    }
    make_frame(&[
        Command::InListPassiveTarget as u8,
        max_tag_number,
        baud_rate,
    ])
}
