---
project: wch-ble-extcap
doc: docs/dev/project/CHANGELOG.md
status: living
last_updated: 2026-07-23
---

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added

- Packaging for Debian/Ubuntu and Raspberry Pi OS (`.deb`, amd64/arm64/armhf),
  Windows (NSIS installer + portable zip), macOS (`.pkg`) and Arch (`PKGBUILD`).
  Every package ships the binary, `LICENSE`, `README.md` and `docs/`
- `packaging/verify.sh`, which reads the file inventory back out of each built
  artifact and exits non-zero if one is incomplete
- udev rule (`udev/60-wch-ble-analyzer.rules`) granting the logged-in user
  access to the dongle, so capturing no longer needs root

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
