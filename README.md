---
project: wch-ble-extcap
doc: README.md
status: living
last_updated: 2026-07-18
---

# WCH BLE Analyzer Pro — Wireshark Extcap Plugin

A Wireshark extcap plugin for the **WCH BLE Analyzer Pro**, written in Rust.  Select it
from Wireshark's capture interface list and start sniffing BLE advertising traffic — no
file saving or piping required.

## Based on

This project is based on the reverse-engineered Linux driver by **Xecaz**:

> [xecaz/BLE-Analyzer-pro-linux-capture](https://github.com/xecaz/BLE-Analyzer-pro-linux-capture)

The USB protocol, frame format, and packet decoding are ported from that C implementation.

---

## Hardware

```
┌─────────────────────────────────────┐
│         WCH BLE Analyzer Pro        │
│                                     │
│  [CH582F ch37]  VID 0x1A86          │
│  [CH582F ch38]  PID 0x8009  × 3     │
│  [CH582F ch39]                      │
│  [CH334 hub  ]  PID 0x8091          │
└─────────────────────────────────────┘
```

Three CH582F MCUs, each assigned to a BLE advertising channel (37 / 38 / 39),
capture the full BLE advertising spectrum simultaneously.

---

## Requirements

- Rust toolchain ([rustup](https://rustup.rs/))
- `libusb-1.0` development headers

```bash
# Debian / Ubuntu
sudo apt install libusb-1.0-0-dev pkg-config
```

---

## Build & install

```bash
cd wireshark-extcap-wch-ble
cargo build --release
sudo cp target/release/wch-ble-extcap /usr/libexec/wireshark/extcap/
```

This installs `wch-ble-extcap` to Wireshark's extcap directory.  Restart Wireshark
and the **WCH BLE Analyzer Pro** will appear in the capture interface list.

> **Note:** The extcap path varies by distribution.  To find yours:
> ```bash
> find / -name ciscodump -type f 2>/dev/null
> ```
> Common locations: `/usr/libexec/wireshark/extcap/` (Debian/Ubuntu),
> `/usr/lib/wireshark/extcap/` (older installs), `/usr/lib64/wireshark/extcap/` (Fedora).

### Uninstall

```bash
sudo rm /usr/libexec/wireshark/extcap/wch-ble-extcap
```

---

## Usage

1. Plug in the WCH BLE Analyzer Pro
2. Open Wireshark or tshark
3. Select **WCH BLE Analyzer Pro** from the interface dropdown
4. Click Start

### Configuration options (in Wireshark)

| Option     | Values                        | Default           |
|------------|-------------------------------|-------------------|
| Channel    | All (auto), 37, 38, 39        | All (auto)        |
| PHY        | 1M, 2M, CodedS8, CodedS2     | 1M                |

### tshark (CLI)

```bash
sudo tshark -i wch-ble-extcap
sudo tshark -i wch-ble-extcap -c 100          # capture 100 packets
sudo tshark -i wch-ble-extcap -w out.pcap     # write to file
```

### Standalone (without Wireshark/tshark)

```bash
# Pipe directly to Wireshark
./target/release/wch-ble-extcap --extcap-capture | wireshark -k -i -

# Write to a pcap file
./target/release/wch-ble-extcap --extcap-capture > capture.pcap
```

### Help

```
./target/release/wch-ble-extcap --help
```

---

## How it works

The extcap interface is the standard mechanism for adding external capture sources to
Wireshark.  When you select the interface, Wireshark launches `wch-ble-extcap` which:

1. Scans USB for WCH BLE Analyzer MCUs (VID `0x1A86` / PID `0x8009`)
2. Opens all three MCUs and sends the init sequence (AA84 → AA81 → AAA1)
3. Reads captured BLE advertising packets from each MCU
4. Writes a `LINKTYPE_BLUETOOTH_LE_LL_WITH_PHDR` (DLT 256) pcap stream to stdout
5. Wireshark decodes and displays the packets in real time

All status and diagnostic messages go to stderr so they don't corrupt the pcap stream.

---

## License

GPL-3.0-only — see [LICENSE](LICENSE).
