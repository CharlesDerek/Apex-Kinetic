# Changelog

## 2026-07-14

### Added

- Added Rust data-plane driver modules for RGB LED, key input, ITR20001 line tracking, voltage sensing, ultrasonic distance, two-axis servo control, and NEC infrared receiver decoding.
- Added expanded DRV8835, TB6612, and MPU6050 driver models based on the legacy robot source.
- Added `docs/drivers.md` with the driver inventory, migration notes, and verification command.

### Changed

- Updated the README with the new data-plane driver component list.
- Removed legacy Chinese comments from the remade Apex driver code and replaced behavior notes with English descriptions.
- Ran Rust formatting on the data plane.

### Verified

- `cargo test` passes for `data-plane`.
- Apex source scan found no non-ASCII characters after the driver migration.
