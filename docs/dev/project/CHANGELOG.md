---
project: wch-ble-extcap
doc: docs/dev/project/CHANGELOG.md
status: living
last_updated: 2026-07-23
---

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.1] - 2026-07-23

### Added

- Packaging for Debian/Ubuntu and Raspberry Pi OS (`.deb`, amd64/arm64/armhf),
  Windows (NSIS installer + portable zip), macOS (`.pkg`) and Arch (`PKGBUILD`).
  Every package ships the binary, `LICENSE`, `README.md` and `docs/`
- `packaging/verify.sh`, which reads the file inventory back out of each built
  artifact and exits non-zero if one is incomplete
- udev rule (`udev/60-wch-ble-analyzer.rules`) granting the logged-in user
  access to the dongle, so capturing no longer needs root
- `scripts/check-version.sh`, checking that Cargo.lock, the PKGBUILD, the
  changelog and the extcap handshake all agree with `Cargo.toml`

### Changed

- The version reported in the extcap handshake is now derived from
  `CARGO_PKG_VERSION` instead of a hardcoded literal, which had already drifted
  and would have reported 0.1.0 from a 0.1.1 build
- Applied `cargo fmt` and cleared the two outstanding clippy lints, so the tree
  now passes `cargo fmt --check` and `cargo clippy -- -D warnings`
- Committed an SPDX 2.3 SBOM (`docs/dev/project/sbom.spdx.json`), regenerated
  each release

### Fixed

- Accept every extcap option Wireshark actually passes: `--extcap-capture-filter`,
  `--extcap-version=X` and the control-pipe options were rejected, which aborted
  the capture pipe. Unknown `--extcap-*` options now warn instead of exiting
- Shut down cleanly on `SIGTERM`. Wireshark stops extcap plugins with SIGTERM,
  but only SIGINT was handled, so stopping a capture killed the process by
  signal and Wireshark reported the non-zero exit as an error dialog
- Gate status output behind `-v`. Wireshark surfaces extcap stderr to the user,
  so a normal run now writes nothing to it

## [0.1.0] - 2026-07-18

### Added

- Wireshark extcap plugin interface (`--extcap-interfaces`, `--extcap-dlts`, `--extcap-config`, `--extcap-capture`)
- USB communication with WCH BLE Analyzer Pro (3x CH582F MCUs)
- PCAP output (DLT 256, BLE LL + pseudo-header)
- Multi-MCU support with automatic channel assignment (ch37/38/39)
- Configurable PHY mode (1M, 2M, CodedS8, CodedS2)
- Configurable advertising channel selection
- Verbose packet logging to stderr (`-v`)
- Ctrl+C graceful shutdown with packet count
- README with hardware documentation and usage instructions

### Fixed

- Use correct extcap output format for interface/DLT discovery (structured `extcap {}` / `interface {}` / `dlt {}` syntax instead of semicolons)

[Unreleased]: https://github.com/dc0sk/wireshark-extcap-wch-ble/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/dc0sk/wireshark-extcap-wch-ble/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/dc0sk/wireshark-extcap-wch-ble/releases/tag/v0.1.0
