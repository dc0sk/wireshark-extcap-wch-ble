---
project: wch-ble-extcap
doc: packaging/README.md
status: living
last_updated: 2026-07-23
---

# Packaging

Build scripts for the distributable formats.  Every package contains the
plugin binary, `LICENSE`, `README.md` and `docs/`; `verify.sh` reads that
inventory back out of the built artifacts rather than trusting a checklist.

All artifacts land in `dist/`.  Intermediate staging trees go to `pkg/`.  Both
are ignored by git.

| Format | Script | Builds on |
|---|---|---|
| Debian/Ubuntu `.deb` (amd64) | `debian/build.sh amd64` | Linux |
| Raspberry Pi OS `.deb` (arm64) | `debian/build.sh arm64` | Linux + cross toolchain |
| Raspberry Pi OS `.deb` (armhf) | `debian/build.sh armhf` | Linux + cross toolchain |
| Windows portable zip | `windows/build.sh` | Linux + mingw-w64 |
| Windows NSIS installer | `windows/build.sh` | Linux + mingw-w64 + nsis |
| macOS `.pkg` | `macos/build.sh` | macOS only |
| Arch `PKGBUILD` | `cd arch && makepkg -si` | Arch/Manjaro |

## Where the plugin is installed

Wireshark's extcap directory is **not** the same across platforms, and getting
it wrong fails silently — the interface simply never appears in Wireshark.

- **Debian family** — the directory follows `libexecdir`, which is
  multiarch-qualified (`/usr/lib/<triple>/wireshark/extcap`) on some releases
  and plain (`/usr/libexec/wireshark/extcap`) on others.  The `.deb` therefore
  installs the real binary to `/usr/bin` and lets `postinst` symlink it into
  whichever directory exists on the target machine.  If none is found,
  `postinst` says so instead of failing quietly.
- **Arch** — `/usr/lib/wireshark/extcap`, stable, so the PKGBUILD writes there
  directly and symlinks `/usr/bin` for `tshark` use.
- **Windows** — read from `HKLM\Software\Wireshark\InstallDir`, falling back to
  `%PROGRAMFILES%\Wireshark\extcap`.
- **macOS** — inside the app bundle, `/Applications/Wireshark.app/Contents/MacOS/extcap`.

## Prerequisites

```bash
# Debian ARM cross-builds (on a Debian host)
rustup target add aarch64-unknown-linux-gnu          # or armv7-unknown-linux-gnueabihf
sudo apt install gcc-aarch64-linux-gnu               # or gcc-arm-linux-gnueabihf

# Windows, cross-compiled from Linux
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64 nsis                      # sudo pacman -S mingw-w64-gcc nsis
```

The `x86_64-pc-windows-msvc` target cannot be built on Linux — it needs
`link.exe`.  Use the `-gnu` target, or build on Windows.

ARM cross-builds also need a `.cargo/config.toml` naming the linker:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

## Verifying

```bash
./packaging/verify.sh        # exits non-zero if any artifact is incomplete
```

Confirm the check can actually fail before trusting a pass — drop a package
into `dist/` with `docs/` removed and make sure it reports `FAIL`.  A verifier
that only ever prints `PASS` is indistinguishable from one that does nothing.

## Release notes

- **`arch/PKGBUILD` carries `sha256sums=('SKIP')`.** Replace it with the real
  digest before publishing — `updpkgsums` does this in place once the release
  tag exists.  `SKIP` disables source verification entirely.
- **The PKGBUILD builds from the GitHub release tag**, so the tag must exist
  and must contain `udev/60-wch-ble-analyzer.rules`; `package()` installs it.
- A `.pkg` on macOS has **no uninstaller**.  `pkgutil --forget
  com.github.dc0sk.wch-ble-extcap` only drops the receipt.  To remove:

  ```bash
  sudo rm /Applications/Wireshark.app/Contents/MacOS/extcap/wch-ble-extcap
  sudo rm /usr/local/bin/wch-ble-extcap
  sudo rm -r /usr/local/share/doc/wch-ble-extcap
  ```

- Signing is not wired up here.  For apt, what clients verify is the signed
  `Release`/`InRelease` metadata of the repository, not a detached signature on
  the `.deb`.
