---
project: wch-ble-extcap
doc: README.md
status: living
last_updated: 2026-07-23
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

## Install from a package

Packages install the plugin into Wireshark's extcap directory, set up the udev
rule, and place `LICENSE`, `README.md` and `docs/` under the platform's usual
documentation path.  Build scripts and per-platform details are in
[`packaging/README.md`](packaging/README.md).

```bash
# Debian / Ubuntu / Raspberry Pi OS
sudo apt install ./wch-ble-extcap_<version>_<arch>.deb    # amd64, arm64, armhf

# Arch / Manjaro
cd packaging/arch && makepkg -si
```

On **Windows**, run the `-setup.exe` installer (it locates Wireshark through the
registry), or unpack the portable zip and copy `wch-ble-extcap.exe` into
Wireshark's `extcap\` directory yourself.  The dongle additionally needs a
WinUSB driver bound to it — [Zadig](https://zadig.akeo.ie/) does this — because
libusb cannot talk to the stock vendor driver.

On **macOS**, run the `.pkg`.  Note it has no uninstaller; `packaging/README.md`
lists the files to remove by hand.

---

## Build & install manually

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

### USB permissions (udev)

The plugin opens the analyzer directly via libusb, but Wireshark runs extcap
plugins as your normal user.  Without a udev rule the device nodes are owned by
`root` and you get:

```
open bus=4 addr=6: Access denied (insufficient permissions)
```

Install the bundled rule:

```bash
sudo cp udev/60-wch-ble-analyzer.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
```

Replug the dongle afterwards if it was already attached.  Verify with:

```bash
lsusb -d 1a86:8009          # should list the analyzer radios
getfacl /dev/bus/usb/004/010    # bus/device numbers from lsusb above
```

`getfacl` must show a `user:<you>:rw-` line.  If it doesn't, the ACL was not
applied — check that the rules file is named `60-…` and not something above
`73-`, since systemd's `73-seat-late.rules` is what runs the `uaccess` builtin.
A tag set after that point shows up in `udevadm info` but has no effect.

> **Note:** The dongle enumerates as **three** separate `1a86:8009` devices
> behind an internal `1a86:8091` hub.  This is normal, and it is what makes the
> default *All channels* setting work: the plugin opens all three and assigns
> advertising channels 37/38/39 one per radio, merging their packet streams.

### Uninstall

```bash
sudo rm /usr/libexec/wireshark/extcap/wch-ble-extcap
sudo rm /etc/udev/rules.d/60-wch-ble-analyzer.rules
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

### Extcap options

The plugin accepts both standard `--extcap-*` options and legacy short forms
for compatibility with older Wireshark versions:

| Standard              | Legacy         | Description                        |
|-----------------------|----------------|------------------------------------|
| `--extcap-interfaces` | `--interfaces` | List available interfaces          |
| `--extcap-dlts`       | `--dlts`       | List DLT types                     |
| `--extcap-interface`  | `--interface`  | Select interface                   |
| `--extcap-config`     | `--config`     | List configuration options         |
| `--extcap-capture`    | `--capture`    | Start capturing                    |
| `--extcap-capture-filter` | `--extcap-filter`, `--filter` | Apply capture filter |
| `--extcap-version`    | —              | Wireshark version, accepted and ignored |

Options Wireshark passes but the plugin does not use (`--extcap-control-in`,
`--extcap-control-out`, `--extcap-reload-option`) are accepted and ignored.  Any
other unrecognised `--extcap-*` option produces a warning rather than an error,
so a future Wireshark release cannot break capture by adding one.

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

Diagnostics never go to stdout, which carries the pcap stream.  They are also
silent by default: Wireshark shows anything an extcap plugin writes to stderr in
an error dialog, so status output is gated behind `-v` and only genuine errors
are printed unconditionally.

---

## License

GPL-3.0-only — see [LICENSE](LICENSE).
