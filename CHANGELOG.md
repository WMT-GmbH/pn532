# Changelog

## [Unreleased]

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
