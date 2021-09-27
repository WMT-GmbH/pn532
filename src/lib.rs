#![feature(const_panic)]
#![feature(generic_associated_types)]
#![no_std]

use core::fmt::Debug;
use core::future::Future;

pub use crate::protocol::{Error, Frame};

mod protocol;
pub mod spi;
pub mod tag;

pub trait Interface {
    type Error: Debug;
    type WaitReadyFuture<'a>: Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;
    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error>;

    /// should be `async fn wait_ready(&mut self) -> Result<(), Self::Error>;`
    fn wait_ready(&mut self) -> Self::WaitReadyFuture<'_>;
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

impl Frame<9> {
    pub const GET_FIRMWARE_VERSION: Frame<9> = Frame::make(&[Command::GetFirmwareVersion as u8]);
}
impl Frame<11> {
    pub const INLIST_ONE_ISO_A_TARGET: Frame<11> = Frame::make(&[
        Command::InListPassiveTarget as u8,
        1,
        CardType::IsoTypeA as u8,
    ]);
}
impl Frame<12> {
    /// Make a SAMConfiguration frame
    pub const fn sam_configuration(mode: SAMMode, use_irq_pin: bool) -> Frame<12> {
        let (mode, timeout) = match mode {
            SAMMode::Normal => (1, 0),
            SAMMode::VirtualCard { timeout } => (2, timeout),
            SAMMode::WiredCard => (3, 0),
            SAMMode::DualCard => (4, 0),
        };
        Frame::make(&[
            Command::SAMConfiguration as u8,
            mode,
            timeout,
            !use_irq_pin as u8,
        ])
    }
}
