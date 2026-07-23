# Shared helpers for the packaging scripts.  Sourced, not executed.
#
# Every package this repo produces must contain the binary, LICENSE, README.md
# and docs/.  Staging goes through stage_docs() so no format can quietly drop
# one of them; packaging/verify.sh re-checks the built artifacts.

PKG_NAME=wch-ble-extcap
BIN_NAME=wch-ble-extcap

# Repo root, regardless of where the script was invoked from.
REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

# Version of the shipped package, never .packages[0] -- that is an arbitrary
# workspace member and would stamp the wrong version on every artifact.
pkg_version() {
    cargo metadata --no-deps --format-version=1 --manifest-path "$REPO_ROOT/Cargo.toml" \
        | jq -r --arg n "$PKG_NAME" '.packages[] | select(.name==$n) | .version'
}

# stage_docs <destination-dir>
# Copies LICENSE, README.md and docs/ into a staging directory.
# docs/ is copied conditionally but not error-swallowing: `if` returns 0 when
# the directory is absent, while a real cp failure still fails the build.
stage_docs() {
    local dest="$1"
    mkdir -p "$dest"
    install -m 644 "$REPO_ROOT/LICENSE" "$dest/LICENSE"
    install -m 644 "$REPO_ROOT/README.md" "$dest/README.md"
    if [ -d "$REPO_ROOT/docs" ]; then cp -r "$REPO_ROOT/docs" "$dest/docs"; fi
}

# build_target <rust-triple>
# --locked so the lockfile is honoured; locking alone is not reproducibility.
build_target() {
    local triple="$1"
    ( cd "$REPO_ROOT" && cargo build --release --locked --target "$triple" )
}

die() { echo "error: $*" >&2; exit 1; }
