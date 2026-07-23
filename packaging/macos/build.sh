#!/usr/bin/env bash
# Build a macOS .pkg.  Must run ON macOS -- pkgbuild/productbuild ship with the
# Xcode Command Line Tools and have no Linux equivalent.
#
# Note: a .pkg has no uninstall mechanism.  "pkgutil --forget" only drops the
# receipt; the files must be removed by hand.  packaging/README.md documents
# the removal commands.
set -euo pipefail

. "$(dirname "${BASH_SOURCE[0]}")/../common.sh"

[ "$(uname -s)" = "Darwin" ] || die "this script must run on macOS"
command -v pkgbuild >/dev/null 2>&1 || die "pkgbuild not found (install the Xcode Command Line Tools)"

VERSION=$(pkg_version)
[ -n "$VERSION" ] || die "could not determine package version"

case "$(uname -m)" in
    arm64) TRIPLE=aarch64-apple-darwin ;;
    *)     TRIPLE=x86_64-apple-darwin ;;
esac

OUT_DIR="$REPO_ROOT/dist"
MAC_DIR="$REPO_ROOT/pkg/macos"
ROOT="$MAC_DIR/root"
# Wireshark.app carries its own extcap directory; that is where the plugin has
# to land for the GUI to see it.  /usr/local/bin additionally covers tshark.
EXTCAP_DIR="$ROOT/Applications/Wireshark.app/Contents/MacOS/extcap"

echo "==> building $PKG_NAME $VERSION for $TRIPLE"
if [ "${SKIP_BUILD:-0}" != 1 ]; then
    build_target "$TRIPLE"
fi
BIN_PATH="$REPO_ROOT/target/$TRIPLE/release/$BIN_NAME"
[ -x "$BIN_PATH" ] || die "binary not found at $BIN_PATH"

rm -rf "$MAC_DIR"
# BSD install has no -D, so the directories are created up front.
mkdir -p "$EXTCAP_DIR" "$ROOT/usr/local/bin" \
         "$ROOT/usr/local/share/doc/$PKG_NAME" "$OUT_DIR"

install -m 755 "$BIN_PATH" "$EXTCAP_DIR/$BIN_NAME"
install -m 755 "$BIN_PATH" "$ROOT/usr/local/bin/$BIN_NAME"
stage_docs "$ROOT/usr/local/share/doc/$PKG_NAME"

pkgbuild --root "$ROOT" \
    --identifier "com.github.dc0sk.$PKG_NAME" \
    --version "$VERSION" \
    --install-location / \
    "$MAC_DIR/$PKG_NAME.pkg"

productbuild --synthesize --package "$MAC_DIR/$PKG_NAME.pkg" "$MAC_DIR/distribution.xml"
productbuild --distribution "$MAC_DIR/distribution.xml" \
    --package-path "$MAC_DIR/" \
    --resources "$MAC_DIR/" \
    "$OUT_DIR/$PKG_NAME-$VERSION-macos.pkg"

echo "==> $OUT_DIR/$PKG_NAME-$VERSION-macos.pkg"
