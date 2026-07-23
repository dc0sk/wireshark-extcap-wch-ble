---
project: wch-ble-extcap
doc: docs/dev/project/RELEASE_NOTES.md
status: living
last_updated: 2026-07-23
---

# Release Notes

Narrative notes per release.  The terse per-change record lives in
[CHANGELOG.md](CHANGELOG.md).

## v0.1.1 — 2026-07-23

The first release that works end to end inside Wireshark. v0.1.0 could be built
and could talk to the hardware, but three separate defects stopped a capture
from actually starting or stopping cleanly in the GUI.

No breaking changes; no migration steps.

### Capture now starts

Wireshark aborts the extcap pipe on any non-zero exit from the plugin, so an
unrecognised command-line option is fatal rather than degrading. Three options
Wireshark really does pass were rejected:

- `--extcap-capture-filter`, sent whenever a capture filter is set. Only the
  non-standard `--extcap-filter` spelling had been handled.
- `--extcap-version=X`, sent on discovery calls. The `=` form never matched.
- `--extcap-control-in` / `--extcap-control-out`, sent with toolbar controls.

Unknown `--extcap-*` options are now a warning rather than a fatal error, so a
future Wireshark release cannot break capture by adding one. A genuinely
malformed argument still exits non-zero.

### Capture now stops without an error dialog

Stopping a capture raised `Error from extcap pipe:` followed by the plugin's
entire startup log. Two independent causes, both fixed:

Wireshark stops an extcap plugin with `SIGTERM`, but the signal handler covered
only `SIGINT`. The process died by signal, never reached its cleanup path, and
Wireshark reported the non-zero exit by dumping whatever the plugin had written
to stderr.

That log should not have existed either. Wireshark surfaces extcap stderr
directly to the user, so device enumeration, the FIFO path and the USB protocol
probes are now behind `-v`. A default run writes nothing to stderr. Genuine
errors — no device found, open or start failures — still print unconditionally.

### Capture no longer needs root

The plugin drives the dongle through libusb, but Wireshark runs extcap plugins
as the invoking user, so the device nodes were unreachable and every radio
failed to open. `udev/60-wch-ble-analyzer.rules` grants access to the
logged-in user.

The `60-` prefix matters: systemd's `73-seat-late.rules` is what runs the
`uaccess` builtin, so a rule numbered above 73 sets the tag too late to have any
effect — and does so silently, since `udevadm info` reports the tag as present
either way. Verify with `getfacl`, not `udevadm info`; the README shows both.

### Packages

Debian/Ubuntu and Raspberry Pi OS (`.deb`, amd64/arm64/armhf), Windows (NSIS
installer and portable zip), macOS (`.pkg`), and an Arch `PKGBUILD`. Each ships
the binary, `LICENSE`, `README.md` and `docs/`; `packaging/verify.sh` reads that
inventory back out of the built artifact.

Wireshark's extcap directory differs per platform and installing to the wrong
one fails silently — the interface just never appears — so the `.deb` resolves
it at install time and the Windows installer reads it from the registry. See
[packaging/README.md](../../../packaging/README.md).

### Verification

Reported by tier, per [evidence-tiers]:

| Claim | Tier | Evidence |
|---|---|---|
| Capture starts, stops cleanly, emits valid pcap | Hardware-in-the-loop | Live capture against the dongle, `SIGTERM`'d as Wireshark does: exit 0, 0 bytes stderr; with `-v`, 259 packets, valid per `capinfos` |
| udev rule grants access | Hardware-in-the-loop | All three radios opened; `getfacl` shows the user ACL |
| `.deb` and Arch package contents | Build artifact inspection | `pacman -Qlp` and the extracted `.deb` payload |
| Windows and macOS packages | **Unbuilt** | Scripts reviewed only; no mingw-w64 and no macOS host available |

Release gate, run locally — there is no CI in this repo, so nothing re-runs
these on the next change:

| Gate | Exit |
|---|---|
| `cargo build --release --locked` | 0 |
| `cargo clippy --release --all-targets -- -D warnings` | 0 |
| `cargo fmt --check` | 0 |
| `scripts/check-version.sh` | 0 |
| `packaging/verify.sh` | 0 |
| `cargo test --release --locked` | 0 — but **0 tests exist**, so this is a vacuous pass and evidence of nothing |

The absent test suite is the weakest point of this release. Every behavioural
claim above rests on manual hardware runs, which no one will repeat
automatically when the code next changes.
