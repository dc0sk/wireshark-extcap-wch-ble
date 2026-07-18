# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

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
