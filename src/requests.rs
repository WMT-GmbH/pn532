#[derive(Copy, Clone)]
pub struct Request<'a> {
    pub command: Command,
    pub data: &'a [u8],
}

#[derive(Copy, Clone)]
pub struct OwnedRequest<const N: usize> {
    pub command: Command,
    pub data: [u8; N],
}

impl<const N: usize> OwnedRequest<N> {
    pub fn borrow(&self) -> Request<'_> {
        Request {
            command: self.command,
            data: &self.data,
        }
    }
}

impl<'a> Request<'a> {
    #[inline]
    pub const fn new(command: Command) -> Self {
        Request { command, data: &[] }
    }
    #[inline]
    pub const fn with_data(mut self, data: &'a [u8]) -> Self {
        self.data = data;
        self
    }

    #[inline]
    pub const fn with_data_array<const N: usize>(self, data: [u8; N]) -> OwnedRequest<N> {
        OwnedRequest {
            command: self.command,
            data,
        }
    }

    pub const GET_FIRMWARE_VERSION: Request<'static> = Request::new(Command::GetFirmwareVersion);
    pub const INLIST_ONE_ISO_A_TARGET: Request<'static> =
        Request::new(Command::InListPassiveTarget).with_data(&[1, CardType::IsoTypeA as u8]);

    pub const SELECT_TAG_1: Request<'static> = Request::new(Command::InSelect).with_data(&[1]);
    pub const SELECT_TAG_2: Request<'static> = Request::new(Command::InSelect).with_data(&[2]);
    pub const DESELECT_TAG_1: Request<'static> = Request::new(Command::InDeselect).with_data(&[1]);
    pub const DESELECT_TAG_2: Request<'static> = Request::new(Command::InDeselect).with_data(&[2]);
    pub const RELEASE_TAG_1: Request<'static> = Request::new(Command::InRelease).with_data(&[1]);
    pub const RELEASE_TAG_2: Request<'static> = Request::new(Command::InRelease).with_data(&[2]);

    pub const fn sam_configuration(mode: SAMMode, use_irq_pin: bool) -> OwnedRequest<3> {
        let (mode, timeout) = match mode {
            SAMMode::Normal => (1, 0),
            SAMMode::VirtualCard { timeout } => (2, timeout),
            SAMMode::WiredCard => (3, 0),
            SAMMode::DualCard => (4, 0),
        };
        Request::new(Command::SAMConfiguration).with_data_array([mode, timeout, !use_irq_pin as u8])
    }

    pub const fn rf_regulation_test(tx_speed: TxSpeed, tx_framing: TxFraming) -> OwnedRequest<1> {
        Request::new(Command::RFRegulationTest).with_data_array([tx_speed as u8 | tx_framing as u8])
    }

    // TODO power down

    pub const fn ntag_read(page: u8) -> OwnedRequest<3> {
        Request::new(Command::InDataExchange).with_data_array([0x01, NTAGCommand::Read as u8, page])
    }
    pub const fn ntag_write(page: u8, bytes: &[u8; 4]) -> OwnedRequest<7> {
        Request::new(Command::InDataExchange).with_data_array([
            0x01,
            NTAGCommand::Write as u8,
            page,
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
        ])
    }
    pub const fn ntag_pwd_auth(bytes: &[u8; 4]) -> OwnedRequest<5> {
        Request::new(Command::InCommunicateThru).with_data_array([
            NTAGCommand::PwdAuth as u8,
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
        ])
    }
}

#[derive(Copy, Clone)]
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
