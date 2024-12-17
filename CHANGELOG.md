# Changelog

## [Unreleased]

## [0.5.0]

### Changed

- migrate to embedded-hal 1.0 #22
- provide our own CountDown trait instead of the one from embedded-hal 0.2.7 #22
- no longer ignore I2C errors in I2CInterface::wait_ready dfc595ec

## [0.4.0]

### Added

- added `BorrowedRequest` #18

### Changed

- changed signatures of `Pn532::process`, `Pn532::process_no_response`, `Pn532::send`, `Pn532::process_async`
  and `Pn532::process_no_response_async` to also allow `BorrowedRequest` parameters #18 (should be mostly backwards
  compatible)

## [0.3.3]

### Added

- Descriptions for the variants of `requests::Command` #16

## [0.3.2]

### Fixed

- fix `I2CInterface` (without Irq pin) `InterfaceError` #14

## [0.3.1]

### Fixed

- tell the async executor to poll until `wait_ready` returns `Ready` #12

## [0.3.0]

### Changed

- renamed some enum variants #3
    - `ErrorCode::Nfcid3InitiatorTargetMismatch` => `ErrorCode::NfcId3InitiatorTargetMismatch`
    - `ErrorCode::NadMsssing` => `ErrorCode::NadMissing`
    - `Error::BadACK` => `Error::BadAck`

### Fixed

- Added missing chip select toggle in SPIInterface (without IRQ) #7

## [0.2.2]

### Added

- Removed now stabilized nightly features
