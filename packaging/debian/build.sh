#!/usr/bin/env bash
# Build a binary .deb for Debian / Ubuntu / Raspberry Pi OS.
#
#   ./packaging/debian/build.sh amd64     # PC
#   ./packaging/debian/build.sh arm64     # Raspberry Pi OS 64-bit (Pi 3/4/5)
#   ./packaging/debian/build.sh armhf     # Raspberry Pi OS 32-bit
#
# Cross-building the ARM variants needs the Rust target plus a cross linker:
#   rustup target add aarch64-unknown-linux-gnu
#   apt install gcc-aarch64-linux-gnu libusb-1.0-0-dev:arm64
# and the matching [target.*] linker entry in .cargo/config.toml.
set -euo pipefail

. "$(dirname "${BASH_SOURCE[0]}")/../common.sh"

DEB_ARCH="${1:-amd64}"
case "$DEB_ARCH" in
    amd64) TRIPLE=x86_64-unknown-linux-gnu ;;
    arm64) TRIPLE=aarch64-unknown-linux-gnu ;;
    armhf) TRIPLE=armv7-unknown-linux-gnueabihf ;;
    *) die "unknown architecture '$DEB_ARCH' (expected amd64, arm64 or armhf)" ;;
esac

VERSION=$(pkg_version)
[ -n "$VERSION" ] || die "could not determine package version"

OUT_DIR="$REPO_ROOT/dist"
PKG_DIR="$REPO_ROOT/pkg/debian-$DEB_ARCH"
DOC_DIR="$PKG_DIR/usr/share/doc/$PKG_NAME"

echo "==> building $PKG_NAME $VERSION for $DEB_ARCH ($TRIPLE)"
if [ "${SKIP_BUILD:-0}" != 1 ]; then
    build_target "$TRIPLE"
fi
BIN_PATH="$REPO_ROOT/target/$TRIPLE/release/$BIN_NAME"
[ -x "$BIN_PATH" ] || die "binary not found at $BIN_PATH"

rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN" "$OUT_DIR"

# Payload.  The binary goes to /usr/bin and postinst symlinks it into the
# extcap directory -- see packaging/debian/postinst for why that is not fixed.
install -Dm755 "$BIN_PATH" "$PKG_DIR/usr/bin/$BIN_NAME"
install -Dm644 "$REPO_ROOT/udev/60-wch-ble-analyzer.rules" \
    "$PKG_DIR/lib/udev/rules.d/60-wch-ble-analyzer.rules"
stage_docs "$DOC_DIR"

# Control metadata.
sed -e "s/@VERSION@/$VERSION/" -e "s/@ARCH@/$DEB_ARCH/" \
    "$REPO_ROOT/packaging/debian/control.in" > "$PKG_DIR/DEBIAN/control"
install -m 755 "$REPO_ROOT/packaging/debian/postinst" "$PKG_DIR/DEBIAN/postinst"
install -m 755 "$REPO_ROOT/packaging/debian/prerm" "$PKG_DIR/DEBIAN/prerm"

DEB="$OUT_DIR/${PKG_NAME}_${VERSION}_${DEB_ARCH}.deb"

if command -v dpkg-deb >/dev/null 2>&1; then
    dpkg-deb --build --root-owner-group "$PKG_DIR" "$DEB"
else
    # Fallback for non-Debian build hosts: a .deb is an ar archive of
    # debian-binary, control.tar.gz and data.tar.gz, in that order.
    echo "==> dpkg-deb not found, assembling the ar archive directly"
    command -v ar >/dev/null 2>&1 || die "neither dpkg-deb nor ar is available"
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    echo "2.0" > "$tmp/debian-binary"
    tar -czf "$tmp/control.tar.gz" --owner=root --group=root --numeric-owner \
        -C "$PKG_DIR/DEBIAN" .
    tar -czf "$tmp/data.tar.gz" --owner=root --group=root --numeric-owner \
        -C "$PKG_DIR" --exclude=./DEBIAN .
    rm -f "$DEB"
    ( cd "$tmp" && ar rc "$DEB" debian-binary control.tar.gz data.tar.gz )
fi

echo "==> $DEB"
