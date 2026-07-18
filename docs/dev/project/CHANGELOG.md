---
project: wch-ble-extcap
doc: docs/dev/project/CHANGELOG.md
status: living
last_updated: 2026-07-18
---

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

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
