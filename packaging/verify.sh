#!/usr/bin/env bash
# Verify every artifact in dist/ actually contains what it is supposed to.
#
# The inventory is read back out of the built artifact, never from a checklist
# kept alongside it -- a checklist agrees with itself no matter what shipped.
# Any artifact missing the binary, LICENSE, README.md or docs/ fails the run.
#
#   ./packaging/verify.sh
set -uo pipefail

. "$(dirname "${BASH_SOURCE[0]}")/../packaging/common.sh"

DIST="$REPO_ROOT/dist"
[ -d "$DIST" ] || die "no dist/ directory -- build something first"

fail=0
checked=0

# check_listing <artifact-label> <listing-text>
check_listing() {
    local label="$1" listing="$2" missing=()
    local -a required=("$BIN_NAME" "LICENSE" "README.md" "docs/")

    for want in "${required[@]}"; do
        grep -qF -- "$want" <<<"$listing" || missing+=("$want")
    done

    if [ ${#missing[@]} -eq 0 ]; then
        printf '  PASS  %s\n' "$label"
    else
        printf '  FAIL  %s -- missing: %s\n' "$label" "${missing[*]}"
        fail=1
    fi
    checked=$((checked + 1))
}

echo "== verifying artifacts in $DIST"

shopt -s nullglob

for deb in "$DIST"/*.deb; do
    if command -v dpkg-deb >/dev/null 2>&1; then
        listing=$(dpkg-deb -c "$deb")
        # Metadata is only inspectable with dpkg-deb; show it for the record.
        dpkg-deb -I "$deb" | grep -E '^ (Package|Version|Architecture):' | sed 's/^/        /'
    else
        # ar + tar fallback so this runs on a non-Debian host too.
        tmp=$(mktemp -d); trap 'rm -rf "$tmp"' RETURN
        ( cd "$tmp" && ar x "$deb" ) || { echo "  FAIL  $(basename "$deb") -- not an ar archive"; fail=1; continue; }
        listing=$(tar -tzf "$tmp"/data.tar.* 2>/dev/null)
        rm -rf "$tmp"
    fi
    check_listing "$(basename "$deb")" "$listing"
done

for zipf in "$DIST"/*.zip; do
    check_listing "$(basename "$zipf")" "$(unzip -l "$zipf")"
done

for pkg in "$DIST"/*.pkg; do
    if command -v pkgutil >/dev/null 2>&1; then
        check_listing "$(basename "$pkg")" "$(pkgutil --payload-files "$pkg")"
    else
        echo "  SKIP  $(basename "$pkg") -- pkgutil unavailable (macOS only)"
    fi
done

for arch_pkg in "$DIST"/*.pkg.tar.zst; do
    if command -v pacman >/dev/null 2>&1; then
        check_listing "$(basename "$arch_pkg")" "$(pacman -Qlp "$arch_pkg")"
    else
        check_listing "$(basename "$arch_pkg")" "$(tar -tf "$arch_pkg")"
    fi
done

for exe in "$DIST"/*setup.exe; do
    if command -v 7z >/dev/null 2>&1; then
        check_listing "$(basename "$exe")" "$(7z l "$exe")"
    else
        echo "  SKIP  $(basename "$exe") -- 7z unavailable, cannot list NSIS payload"
    fi
done

if [ "$checked" -eq 0 ]; then
    die "no artifacts were checked -- dist/ contains nothing recognisable"
fi

echo "== $checked artifact(s) checked"
[ "$fail" -eq 0 ] && echo "== OK" || echo "== FAILED"
exit "$fail"
