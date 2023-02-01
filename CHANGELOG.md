# Changelog

## [Unreleased]

### Added

### Changed

- renamed some enum variants #3
  - `ErrorCode::Nfcid3InitiatorTargetMismatch` => `ErrorCode::NfcId3InitiatorTargetMismatch`
  - `ErrorCode::NadMsssing` => `ErrorCode::NadMissing`
  - `Error::BadACK` => `Error::BadAck`

### Fixed

- Added missing chip select toggle in SPIInterface (without IRQ) #7
- tell the async excecutor to poll until `wait_ready` returns `Ready` #12

## [0.2.2]

### Added
- Removed now stabilized nightly features

### Changed

### Fixed
