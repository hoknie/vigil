#!/usr/bin/env bash

set -euo pipefail

cd "$(dirname "$0")/../.."
ROOT="$PWD"

OUT="${OUT:-$ROOT/dist}"
RELEASE="${RELEASE:-$OUT/release}"
TEMPLATE="$ROOT/packaging/homebrew/vigil.rb"
TAP_README="$ROOT/packaging/homebrew/README.md"
LOCAL_TAP="vigil-check/local"
REPOSITORY="${REPOSITORY:-hoknie/vigil}"

VERSION="${VERSION:-$(awk '/^\[workspace\.package\]/{p=1} p && /^version *= *"/{gsub(/[^0-9A-Za-z.+~-]/,"",$3); print $3; exit}' Cargo.toml)}"
[ -n "$VERSION" ] || { echo "homebrew: cannot read the version out of Cargo.toml" >&2; exit 2; }

ARCHIVE_NAME="vigil_${VERSION}.macos.universal.tar.gz"
WORK=""

say() { printf '\n\033[1m== %s\033[0m\n' "$*"; }
die() { printf 'homebrew: %s\n' "$*" >&2; exit 1; }

sum() {
    if command -v sha256sum >/dev/null 2>&1; then sha256sum "$@"; else shasum -a 256 "$@"; fi
}

digest_of() {
    sum "$1" | awk '{ print $1 }'
}

listed_digest() {
    awk -v name="$ARCHIVE_NAME" '{ file = $2; sub(/^\*?\.?\//, "", file) } file == name { print $1; exit }' "$RELEASE/SHA256SUMS"
}

render() {
    local url="$1" digest="$2" into="$3"
    mkdir -p "$(dirname "$into")"
    sed -e "s#@URL@#$url#" -e "s#@VERSION@#$VERSION#" -e "s#@SHA256@#$digest#" "$TEMPLATE" > "$into"
    if grep -n '@[A-Z0-9]*@' "$into"; then
        die "a label is left in $into"
    fi
}

formula() {
    local base="${1:-https://github.com/$REPOSITORY/releases/download/v$VERSION}"
    local archive="$RELEASE/$ARCHIVE_NAME"
    [ -f "$archive" ] || die "$archive is missing: run 'env/scripts/package-macos.sh all' and 'env/scripts/release.sh collect' first"
    local actual listed=""
    actual="$(digest_of "$archive")"
    if [ -f "$RELEASE/SHA256SUMS" ]; then
        listed="$(listed_digest)"
        [ -n "$listed" ] || die "SHA256SUMS does not list $ARCHIVE_NAME"
        [ "$listed" = "$actual" ] || die "SHA256SUMS says $listed for $ARCHIVE_NAME and the file is $actual"
    fi
    render "${base%/}/$ARCHIVE_NAME" "$actual" "$RELEASE/vigil.rb"
    echo "$RELEASE/vigil.rb"
}

check() {
    [ "$(uname -s)" = Darwin ] || die "the formula is installed with brew, which this check runs only on macOS"
    command -v brew >/dev/null 2>&1 || die "brew is not installed"
    local archive="$OUT/$ARCHIVE_NAME"
    [ -f "$archive" ] || archive="$RELEASE/$ARCHIVE_NAME"
    [ -f "$archive" ] || die "$ARCHIVE_NAME is missing: run 'env/scripts/package-macos.sh all' first"

    local work; work="$(mktemp -d)"; WORK="$work"
    cleanup() {
        brew uninstall --formula --force "$LOCAL_TAP/vigil" >/dev/null 2>&1 || true
        brew untap "$LOCAL_TAP" >/dev/null 2>&1 || true
        [ -z "$WORK" ] || rm -rf "$WORK"
    }
    trap cleanup EXIT

    say "a local tap holding the formula of this build"
    brew untap "$LOCAL_TAP" >/dev/null 2>&1 || true
    brew tap-new --no-git "$LOCAL_TAP" >/dev/null
    local tap; tap="$(brew --repository "$LOCAL_TAP")"
    cp "$archive" "$work/$ARCHIVE_NAME"
    render "file://$work/$ARCHIVE_NAME" "$(digest_of "$archive")" "$tap/Formula/vigil.rb"

    say "brew install"
    HOMEBREW_NO_AUTO_UPDATE=1 HOMEBREW_NO_INSTALL_FROM_API=1 brew install --formula "$LOCAL_TAP/vigil"

    local prefix; prefix="$(brew --prefix "$LOCAL_TAP/vigil")"
    for program in sbin/vigild bin/vigil; do
        [ -x "$prefix/$program" ] || die "$prefix/$program is not installed"
    done
    [ -f "$prefix/share/vigil/defaults/vigil.yaml" ] || die "the defaults are not installed under share/vigil"

    say "brew test"
    brew test "$LOCAL_TAP/vigil"

    say "brew audit --strict"
    HOMEBREW_NO_INSTALL_FROM_API=1 brew audit --strict --formula "$LOCAL_TAP/vigil"

    say "brew uninstall"
    brew uninstall --formula "$LOCAL_TAP/vigil"
    [ ! -e "$prefix" ] || die "$prefix is left after the uninstall"
    echo "the formula installs, tests, audits and uninstalls cleanly"
}

publish() {
    local tap="${1:-}"
    [ -d "$tap" ] || die "usage: env/scripts/homebrew.sh publish <tap-directory>"
    [ -f "$RELEASE/vigil.rb" ] || die "$RELEASE/vigil.rb is missing: run 'env/scripts/homebrew.sh formula' first"
    mkdir -p "$tap/Formula"
    cp "$RELEASE/vigil.rb" "$tap/Formula/vigil.rb"
    cp "$TAP_README" "$tap/README.md"
    echo "$tap/Formula/vigil.rb"
}

tap() {
    local tap="${1:-}"
    [ -d "$tap/.git" ] || die "usage: env/scripts/homebrew.sh tap <clone-of-the-tap>"
    local base="https://github.com/$REPOSITORY/releases/download/v$VERSION"
    local fetched; fetched="$(mktemp -d)"; WORK="$fetched"
    trap '[ -z "$WORK" ] || rm -rf "$WORK"' EXIT

    say "the tap, up to date"
    git -C "$tap" pull --ff-only --quiet || die "git pull in $tap failed: see the line above"

    say "the archive and its checksums of v$VERSION"
    curl -fsSL --retry 3 -o "$fetched/SHA256SUMS" "$base/SHA256SUMS" \
        || die "v$VERSION has no SHA256SUMS at $base: is that release published?"
    curl -fsSL --retry 3 -o "$fetched/$ARCHIVE_NAME" "$base/$ARCHIVE_NAME" \
        || die "v$VERSION has no $ARCHIVE_NAME at $base"

    say "the formula of v$VERSION in $tap"
    RELEASE="$fetched"
    formula "$base" >/dev/null
    publish "$tap"
    git -C "$tap" status --short
}

case "${1:-}" in
formula) shift; formula "$@" ;;
check) check ;;
publish) shift; publish "$@" ;;
tap) shift; tap "$@" ;;
*)
    echo "usage: env/scripts/homebrew.sh {formula [url-base]|check|publish <tap-directory>|tap <clone-of-the-tap>}" >&2
    exit 2
    ;;
esac
