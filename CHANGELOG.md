# Changelog

## [Unreleased]

### Added

- Removed now stabilized nightly features

### Changed

- renamed some enum variants #3
  - `ErrorCode::Nfcid3InitiatorTargetMismatch` => `ErrorCode::NfcId3InitiatorTargetMismatch`
  - `ErrorCode::NadMsssing` => `ErrorCode::NadMissing`
  - `Error::BadACK` => `Error::BadAck`

### Fixed

- Added missing chip select toggle in SPIInterface (without IRQ) #7
