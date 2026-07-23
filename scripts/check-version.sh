#!/usr/bin/env bash
# Verify every place the release version appears agrees with Cargo.toml.
#
# The usual single-crate check --
#   grep -rh '^version = ' --include=Cargo.toml . | sort | uniq -c
# -- is degenerate in this repo: there is exactly one manifest, so it compares
# a file against itself and can never fail.  The version also reaches the
# PKGBUILD and, at runtime, the extcap handshake Wireshark parses.  Those are
# the places that actually drift, so those are what this checks.
#
#   ./scripts/check-version.sh
set -uo pipefail

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$REPO_ROOT"

fail=0
report() {  # report <label> <found> <expected>
    if [ "$2" = "$3" ]; then
        printf '  PASS  %-28s %s\n' "$1" "$2"
    else
        printf '  FAIL  %-28s %s (expected %s)\n' "$1" "${2:-<not found>}" "$3"
        fail=1
    fi
}

EXPECTED=$(cargo metadata --no-deps --format-version=1 \
    | jq -r '.packages[] | select(.name=="wch-ble-extcap") | .version')
[ -n "$EXPECTED" ] || { echo "error: could not read version from Cargo.toml" >&2; exit 2; }

echo "== version consistency (expected $EXPECTED)"

report "Cargo.lock" \
    "$(awk '/^name = "wch-ble-extcap"$/{getline; gsub(/version = |"/,""); print; exit}' Cargo.lock)" \
    "$EXPECTED"

report "packaging/arch/PKGBUILD" \
    "$(sed -n 's/^pkgver=//p' packaging/arch/PKGBUILD)" \
    "$EXPECTED"

# The extcap handshake Wireshark reads.  Built from source so a stale binary
# cannot make this pass.
cargo build --release --locked >/dev/null 2>&1 || { echo "error: build failed" >&2; exit 2; }
report "extcap handshake" \
    "$(./target/release/wch-ble-extcap --extcap-interfaces \
        | sed -n 's/^extcap {version=\(.*\)}$/\1/p')" \
    "$EXPECTED"

report "CHANGELOG.md section" \
    "$(sed -n 's/^## \[\([0-9][^]]*\)\].*/\1/p' docs/dev/project/CHANGELOG.md | head -1)" \
    "$EXPECTED"

[ "$fail" -eq 0 ] && echo "== OK" || echo "== FAILED"
exit "$fail"
