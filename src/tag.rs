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
