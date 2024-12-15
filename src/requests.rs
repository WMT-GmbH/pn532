//! Pn532 Requests

/// Pn532 Request consisting of a [`Command`] and extra command data
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Request<const N: usize> {
    pub command: Command,
    pub data: [u8; N],
}

/// Pn532 Request consisting of a [`Command`] and a reference to extra command data
pub struct BorrowedRequest<'a> {
    pub command: Command,
    pub data: &'a [u8],
}

impl<'a, const N: usize> From<&'a Request<N>> for BorrowedRequest<'a> {
    fn from(value: &'a Request<N>) -> BorrowedRequest<'a> {
        BorrowedRequest::new(value.command, &value.data)
    }
}

impl<const N: usize> Request<N> {
    #[inline]
    pub const fn new(command: Command, data: [u8; N]) -> Self {
        Request { command, data }
    }
}

impl<'a> BorrowedRequest<'a> {
    #[inline]
    pub const fn new(command: Command, data: &'a [u8]) -> Self {
        Self { command, data }
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
///
/// These commands are fully described in the section 7 of the User Manual:
/// <https://www.nxp.com/docs/en/user-guide/141520.pdf>
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum Command {
    /// This command is used for self-diagnosis. Processing time of this command varies depending
    /// on the content of the processing.
    ///
    /// For more information, see 7.2.1 Diagnose
    Diagnose = 0x00,
    /// This command is used to get the version of the embedded firmware from PN532.
    ///
    /// For more information, see 7.2.2 GetFirmwareVersion
    GetFirmwareVersion = 0x02,
    /// This command allows the host controller to know at a given moment the complete situation of
    /// the PN532.
    ///
    /// For more information, see 7.2.3 GetGeneralStatus
    GetGeneralStatus = 0x04,
    /// This command is used to read the content of one or several internal registers of the PN532.
    ///
    /// For more information, see 7.2.4 ReadRegister
    ReadRegister = 0x06,
    /// This command is used to overwrite the content of one or several internal registers of the
    /// PN532.
    ///
    /// For more information, see 7.2.5 WriteRegister
    WriteRegister = 0x08,
    /// Tells the PN532 to read the value for each GPIO port and return the information to the host
    /// controller.
    ///
    /// For more information, see 7.2.6 ReadGPIO
    ReadGPIO = 0x0C,
    /// Tells the PN532 to apply the value for each port specified by the host controller.
    ///
    /// For more information, see 7.2.7 WriteGPIO
    WriteGPIO = 0x0E,
    /// Selects the baud rate on the serial link between the host controller and the PN532.
    ///
    /// For more information, see 7.2.8 SetSerialBaudRate
    SetSerialBaudRate = 0x10,
    /// This command is used to set internal parameters of the PN532, and then to configure its
    /// behavior regarding different cases.
    ///
    /// For more information, see 7.2.9 SetParameters
    SetParameters = 0x12,
    /// This command is used to select the data flow path by configuring the internal serial data
    /// switch.
    ///
    /// For more information, see 7.2.10 SAMConfiguration
    SAMConfiguration = 0x14,
    /// This command can be used to put the PN532 into Power Down mode in order to save power
    /// consumption.
    ///
    /// For more information, see 7.2.11 PowerDown
    PowerDown = 0x16,
    /// This command is used to configure the different settings of the PN532.
    ///
    /// For more information, see 7.3.1 RFConfiguration
    RFConfiguration = 0x32,
    /// This command is used for radio regulation test.
    ///
    /// For more information, see 7.3.2 RFRegulationTest
    RFRegulationTest = 0x58,
    /// This command is used by a host controller to activate a target using either active or
    /// passive communication mode during communication over DEP protocol.
    ///
    /// For more information, see 7.3.3 InJumpForDEP
    InJumpForDEP = 0x56,
    /// This command is used by a host controller to activate a target using either active or
    /// passive communication mode during communication over PSL or DEP protocols.
    ///
    /// For more information, see 7.3.4 InJumpForPSL
    InJumpForPSL = 0x46,
    /// This command tells PN532 to detect as many targets as possible in passive mode.
    ///
    /// For more information, see 7.3.5 InListPassiveTarget
    InListPassiveTarget = 0x4A,
    /// This command is used by a host controller to launch an activation of a target in case of
    /// passive mode.
    ///
    /// For more information, see 7.3.6 InATR
    InATR = 0x50,
    /// This command is used by a host controller to change the defined bit rates either with a TPE
    /// target or with a ISO/IEC14443-4 target.
    ///
    /// For more information, see 7.3.7 InPSL
    InPSL = 0x4E,
    /// This command is used to support protocol data exchanges between the PN532 as initiator and
    /// a target.
    ///
    /// For more information, see 7.3.8 InDataExchange
    InDataExchange = 0x40,
    /// This command is used to support basic data exchanges between the PN532 and a target.
    ///
    /// For more information, see 7.3.9 InCommunicateThru
    InCommunicateThru = 0x42,
    /// Command to deselect specified targets(s).
    ///
    /// For more information, see 7.3.10 InDeselect
    InDeselect = 0x44,
    /// Command to release the specified target(s).
    ///
    /// For more information, see 7.3.11 InRelease
    InRelease = 0x52,
    /// Command to select the specified target.
    ///
    /// For more information, see 7.3.12 InSelect
    InSelect = 0x54,
    /// This command is used to poll card(s) / target(s) of specified Type present in the RF field.
    ///
    /// For more information, see 7.3.13 InAutoPoll
    InAutoPoll = 0x60,
    /// The host controller uses this command to configure the PN532 as target.
    ///
    /// For more information, see 7.3.14 TgInitAsTarget
    TgInitAsTarget = 0x8C,
    /// This command is used to give General Bytes to the PN532.
    ///
    /// For more information, see 7.3.15 TgSetGeneralBytes
    TgSetGeneralBytes = 0x92,
    /// This command allows the host controller to get back the data received by the PN532 from its
    /// initiator.
    ///
    /// For more information, see 7.3.16 TgGetData
    TgGetData = 0x86,
    /// This command allows the host controller to spully PN532 with teh data that it wants to send
    /// back to teh initiator.
    ///
    /// For more information, see 7.3.17 TgSetData
    TgSetData = 0x8E,
    /// This command is used if the overall amount of data to be sent cannot be transmitted in one
    /// frame.
    ///
    /// For more information, see 7.3.18 TgSetMetaData
    TgSetMetaData = 0x94,
    /// This command is used to get a packet of data from an initiator and to send it back to the
    /// host controller.
    ///
    /// For more information, see 7.3.19 TgGetInitiatorCommand
    TgGetInitiatorCommand = 0x88,
    /// This command is used to send a response packet of data to an initiator.
    ///
    /// For more information, see 7.3.20 TgResponseToInitiator
    TgResponseToInitiator = 0x90,
    /// This command is used by the host controller to know what the current state of the PN532 is.
    ///
    /// For more information, see 7.3.21 TgGetTargetStatus
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
