#!/usr/bin/env bash
# Build the Windows artifacts: a portable zip, and an NSIS installer if
# makensis is available.
#
# Cross-compiling from Linux requires the GNU toolchain:
#   rustup target add x86_64-pc-windows-gnu
#   apt install mingw-w64 nsis        # pacman -S mingw-w64-gcc nsis
#
# The *-msvc target cannot be built on Linux -- it fails with
# "linker `link.exe` not found".  MSVC means building on Windows, or cargo-xwin.
set -euo pipefail

. "$(dirname "${BASH_SOURCE[0]}")/../common.sh"

TRIPLE=x86_64-pc-windows-gnu
VERSION=$(pkg_version)
[ -n "$VERSION" ] || die "could not determine package version"

OUT_DIR="$REPO_ROOT/dist"
STAGE="$REPO_ROOT/pkg/portable-$PKG_NAME-$VERSION"

echo "==> building $PKG_NAME $VERSION for $TRIPLE"
if [ "${SKIP_BUILD:-0}" != 1 ]; then
    build_target "$TRIPLE"
fi
EXE="$REPO_ROOT/target/$TRIPLE/release/$BIN_NAME.exe"
[ -f "$EXE" ] || die "binary not found at $EXE"

# --- portable zip: no installer, no registry ------------------------------
rm -rf "$STAGE"
mkdir -p "$STAGE" "$OUT_DIR"
install -m 755 "$EXE" "$STAGE/$BIN_NAME.exe"
stage_docs "$STAGE"

ZIP="$OUT_DIR/$PKG_NAME-$VERSION-windows-x64.zip"
rm -f "$ZIP"
( cd "$REPO_ROOT/pkg" && zip -qr "$ZIP" "portable-$PKG_NAME-$VERSION" )
echo "==> $ZIP"

# --- NSIS installer -------------------------------------------------------
if command -v makensis >/dev/null 2>&1; then
    makensis -DVERSION="$VERSION" "$REPO_ROOT/packaging/windows/installer.nsi"
    echo "==> $OUT_DIR/$PKG_NAME-$VERSION-windows-x64-setup.exe"
else
    echo "==> makensis not found, skipping the installer (portable zip built)" >&2
fi
