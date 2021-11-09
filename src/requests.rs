//! Pn532 Requests

/// Pn532 Request consisting of a [`Command`] and extra command data
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Request<const N: usize> {
    pub command: Command,
    pub data: [u8; N],
}

pub(crate) struct BorrowedRequest<'a> {
    pub command: Command,
    pub data: &'a [u8],
}

impl<const N: usize> Request<N> {
    pub(crate) fn borrow(&self) -> BorrowedRequest<'_> {
        BorrowedRequest {
            command: self.command,
            data: &self.data,
        }
    }
}

impl<const N: usize> Request<N> {
    #[inline]
    pub const fn new(command: Command, data: [u8; N]) -> Self {
        Request { command, data }
    }
}

impl Request<0> {
    pub const GET_FIRMWARE_VERSION: Request<0> = Request::new(Command::GetFirmwareVersion, []);
    pub const INLIST_ONE_ISO_A_TARGET: Request<2> =
        Request::new(Command::InListPassiveTarget, [1, CardType::IsoTypeA as u8]);

    pub const SELECT_TAG_1: Request<1> = Request::new(Command::InSelect, [1]);
    pub const SELECT_TAG_2: Request<1> = Request::new(Command::InSelect, [2]);
    pub const DESELECT_TAG_1: Request<1> = Request::new(Command::InDeselect, [1]);
    pub const DESELECT_TAG_2: Request<1> = Request::new(Command::InDeselect, [2]);
    pub const RELEASE_TAG_1: Request<1> = Request::new(Command::InRelease, [1]);
    pub const RELEASE_TAG_2: Request<1> = Request::new(Command::InRelease, [2]);

    pub const fn sam_configuration(mode: SAMMode, use_irq_pin: bool) -> Request<3> {
        // TODO use_irq_pin seems to not have any effect
        let (mode, timeout) = match mode {
            SAMMode::Normal => (1, 0),
            SAMMode::VirtualCard { timeout } => (2, timeout),
            SAMMode::WiredCard => (3, 0),
            SAMMode::DualCard => (4, 0),
        };
        Request::new(
            Command::SAMConfiguration,
            [mode, timeout, use_irq_pin as u8],
        )
    }

    pub const fn rf_regulation_test(tx_speed: TxSpeed, tx_framing: TxFraming) -> Request<1> {
        Request::new(
            Command::RFRegulationTest,
            [tx_speed as u8 | tx_framing as u8],
        )
    }

    pub const fn ntag_read(page: u8) -> Request<3> {
        Request::new(
            Command::InDataExchange,
            [0x01, NTAGCommand::Read as u8, page],
        )
    }
    pub const fn ntag_write(page: u8, bytes: &[u8; 4]) -> Request<7> {
        Request::new(
            Command::InDataExchange,
            [
                0x01,
                NTAGCommand::Write as u8,
                page,
                bytes[0],
                bytes[1],
                bytes[2],
                bytes[3],
            ],
        )
    }
    pub const fn ntag_pwd_auth(bytes: &[u8; 4]) -> Request<5> {
        Request::new(
            Command::InCommunicateThru,
            [
                NTAGCommand::PwdAuth as u8,
                bytes[0],
                bytes[1],
                bytes[2],
                bytes[3],
            ],
        )
    }
}

/// Commands supported by the Pn532
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum Command {
    /// See 7.2.1 Diagnose
    Diagnose = 0x00,
    /// See 7.2.2 GetFirmwareVersion
    GetFirmwareVersion = 0x02,
    /// See 7.2.3 GetGeneralStatus
    GetGeneralStatus = 0x04,
    /// See 7.2.4 ReadRegister
    ReadRegister = 0x06,
    /// See 7.2.5 WriteRegister
    WriteRegister = 0x08,
    /// See 7.2.6 ReadGPIO
    ReadGPIO = 0x0C,
    /// See 7.2.7 WriteGPIO
    WriteGPIO = 0x0E,
    /// See 7.2.8 SetSerialBaudRate
    SetSerialBaudRate = 0x10,
    /// See 7.2.9 SetParameters
    SetParameters = 0x12,
    /// See 7.2.10 SAMConfiguration
    SAMConfiguration = 0x14,
    /// See 7.2.11 PowerDown
    PowerDown = 0x16,
    /// See 7.3.1 RFConfiguration
    RFConfiguration = 0x32,
    /// See 7.3.2 RFRegulationTest
    RFRegulationTest = 0x58,
    /// See 7.3.3 InJumpForDEP
    InJumpForDEP = 0x56,
    /// See 7.3.4 InJumpForPSL
    InJumpForPSL = 0x46,
    /// See 7.3.5 InListPassiveTarget
    InListPassiveTarget = 0x4A,
    /// See 7.3.6 InATR
    InATR = 0x50,
    /// See 7.3.7 InPSL
    InPSL = 0x4E,
    /// See 7.3.8 InDataExchange
    InDataExchange = 0x40,
    /// See 7.3.9 InCommunicateThru
    InCommunicateThru = 0x42,
    /// See 7.3.10 InDeselect
    InDeselect = 0x44,
    /// See 7.3.11 InRelease
    InRelease = 0x52,
    /// See 7.3.12 InSelect
    InSelect = 0x54,
    /// See 7.3.13 InAutoPoll
    InAutoPoll = 0x60,
    /// See 7.3.14 TgInitAsTarget
    TgInitAsTarget = 0x8C,
    /// See 7.3.15 TgSetGeneralBytes
    TgSetGeneralBytes = 0x92,
    /// See 7.3.16 TgGetData
    TgGetData = 0x86,
    /// See 7.3.17 TgSetData
    TgSetData = 0x8E,
    /// See 7.3.18 TgSetMetaData
    TgSetMetaData = 0x94,
    /// See 7.3.19 TgGetInitiatorCommand
    TgGetInitiatorCommand = 0x88,
    /// See 7.3.20 TgResponseToInitiator
    TgResponseToInitiator = 0x90,
    /// See 7.3.21 TgGetTargetStatus
    TgGetTargetStatus = 0x8A,
}

/// SAM mode parameter to be used in [`Command::SAMConfiguration`]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

/// Card type parameter to be used in [`Command::InListPassiveTarget`]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

/// Bitrate to be used in [`Command::RFRegulationTest`]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum TxSpeed {
    /// 106 kbps
    Tx106kbps = 0b0000_0000,
    /// 212 kbps
    Tx212kbps = 0b0001_0000,
    /// 424 kbps
    Tx424kbps = 0b0010_0000,
    /// 848 kbps
    Tx848kbps = 0b0011_0000,
}

/// Type of modulation to be used in [`Command::RFRegulationTest`]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum TxFraming {
    Mifare = 0b0000_0000,
    FeliCa = 0b0000_0010,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum NTAGCommand {
    GetVersion = 0x60,
    Read = 0x30,
    FastRead = 0x3A,
    Write = 0xA2,
    CompWrite = 0xA0,
    ReadCnt = 0x39,
    PwdAuth = 0x1B,
    ReadSig = 0x3C,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum MifareCommand {
    AuthenticationWithKeyA = 0x60,
    AuthenticationWithKeyB = 0x61,
    PersonalizeUIDUsage = 0x40,
    SetModType = 0x43,
    Read = 0x30,
    Write = 0xA0,
    Decrement = 0xC0,
    Increment = 0xC1,
    Restore = 0xC2,
    Transfer = 0xB0,
}
