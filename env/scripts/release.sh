#!/usr/bin/env bash

set -euo pipefail

cd "$(dirname "$0")/../.."
ROOT="$PWD"

TARGET="${TARGET:-x86_64-unknown-linux-musl}"
OUT="${OUT:-$ROOT/dist}"
RELEASE="${RELEASE:-$OUT/release}"

VERSION="${VERSION:-$(awk '/^\[workspace\.package\]/{p=1} p && /^version *= *"/{gsub(/[^0-9A-Za-z.+~-]/,"",$3); print $3; exit}' Cargo.toml)}"
[ -n "$VERSION" ] || { echo "release: cannot read the version out of Cargo.toml" >&2; exit 2; }

case "$TARGET" in
*-apple-darwin) DEB_ARCH=none; RPM_ARCH=none; BIN_ARCH=universal ;;
x86_64-*)  DEB_ARCH=amd64; RPM_ARCH=x86_64; BIN_ARCH=x86_64 ;;
aarch64-*) DEB_ARCH=arm64; RPM_ARCH=aarch64; BIN_ARCH=aarch64 ;;
*) echo "release: no architecture name known for $TARGET" >&2; exit 2 ;;
esac

BINARIES="$OUT/bin/$TARGET"

say() { printf '\n\033[1m== %s\033[0m\n' "$*"; }

sum() {
    if command -v sha256sum >/dev/null 2>&1; then sha256sum "$@"; else shasum -a 256 "$@"; fi
}

check_tag() {
    local tag="${1:-}"
    [ -n "$tag" ] || { echo "release: usage: env/scripts/release.sh check-tag vX.Y.Z" >&2; exit 2; }
    tag="${tag#refs/tags/}"

    if [ "$tag" != "v$VERSION" ]; then
        echo "release: the tag is $tag and Cargo.toml says $VERSION." >&2
        echo "release: the packages would carry a version the binaries do not print." >&2
        echo "release: set version = \"${tag#v}\" in [workspace.package] and tag that commit." >&2
        exit 1
    fi
    echo "$VERSION"
}

build_archive() {
    [ -x "$BINARIES/vigild" ] || {
        echo "release: $BINARIES/vigild is missing — run 'env/scripts/package.sh binaries' first" >&2
        exit 2
    }

    local name="vigil_${VERSION}.linux.${BIN_ARCH}"
    local work; work="$(mktemp -d)"
    local tree="$work/$name"

    mkdir -p "$tree"
    install -m 0755 "$BINARIES/vigild" "$tree/vigild"
    install -m 0755 "$BINARIES/vigil" "$tree/vigil"
    install -m 0755 "$BINARIES/vigil-audit-plugin" "$tree/vigil-audit-plugin"
    install -m 0755 "$BINARIES/vigil-container-dump" "$tree/vigil-container-dump"
    install -m 0644 "$ROOT/config/vigil.example.yaml" "$tree/vigil.example.yaml"
    mkdir -p "$tree/collectors"
    for collectors in "$ROOT"/config/collectors/*.yaml; do
        install -m 0644 "$collectors" "$tree/collectors/$(basename "$collectors")"
    done
    install -m 0644 "$ROOT/config/watch_fs.yaml" "$tree/watch_fs.yaml"
    install -m 0644 "$ROOT/packaging/systemd/vigild.service" "$tree/vigild.service"
    install -m 0644 "$ROOT/packaging/systemd/vigil-containers.service" "$tree/vigil-containers.service"
    install -m 0644 "$ROOT/packaging/systemd/vigil-containers.timer" "$tree/vigil-containers.timer"
    install -m 0644 "$ROOT/packaging/systemd/vigil-tmpfiles.conf" "$tree/vigil-tmpfiles.conf"
    install -m 0644 "$ROOT/README.md" "$tree/README.md"
    install -m 0644 "$ROOT/LICENSE" "$tree/LICENSE"
    install -m 0644 "$ROOT/NOTICE" "$tree/NOTICE"
    install -m 0644 "$ROOT/THIRD-PARTY.md" "$tree/THIRD-PARTY.md"

    mkdir -p "$RELEASE"
    local archive="$RELEASE/$name.tar.gz"
    rm -f "$archive"

    if tar --version 2>/dev/null | grep -q GNU; then
        tar --owner=0 --group=0 --numeric-owner --sort=name --mtime=@0 \
            -C "$work" -cf - "$name" | gzip -9 -n > "$archive"
    else
        tar -C "$work" -cf - "$name" | gzip -9 -n > "$archive"
    fi
    rm -rf "$work"

    tar -tzf "$archive"
    echo "$archive"
}

collect_macos() {
    local pkg="$OUT/vigil_${VERSION}.macos.universal.pkg"
    local archive="$OUT/vigil_${VERSION}.macos.universal.tar.gz"

    for built in "$pkg" "$archive"; do
        [ -f "$built" ] || {
            echo "release: $built is missing — run 'just package-macos' on a Mac first" >&2
            exit 2
        }
    done

    say "the assets for macOS"
    mkdir -p "$RELEASE"
    install -m 0644 "$pkg" "$RELEASE/$(basename "$pkg")"
    install -m 0644 "$archive" "$RELEASE/$(basename "$archive")"

    ls -l "$RELEASE"
}

collect() {
    case "$TARGET" in
    *-apple-darwin) collect_macos; return ;;
    esac

    local deb="$OUT/vigil_${VERSION}_${DEB_ARCH}.deb"
    local rpm="$OUT/vigil-${VERSION}-1.${RPM_ARCH}.rpm"

    for built in "$deb" "$rpm"; do
        [ -f "$built" ] || {
            echo "release: $built is missing — run 'just package' for TARGET=$TARGET first" >&2
            exit 2
        }
    done

    say "the assets for $TARGET"
    mkdir -p "$RELEASE"
    install -m 0644 "$deb" "$RELEASE/vigil_${VERSION}.debian.${DEB_ARCH}.deb"
    install -m 0644 "$rpm" "$RELEASE/vigil_${VERSION}.el.${RPM_ARCH}.rpm"
    build_archive >/dev/null

    ls -l "$RELEASE"
}

checksums() {
    [ -d "$RELEASE" ] || { echo "release: $RELEASE does not exist" >&2; exit 2; }
    cd "$RELEASE"
    rm -f SHA256SUMS

    local assets=()
    while IFS= read -r asset; do assets+=("${asset#./}"); done \
        < <(find . -maxdepth 1 -type f ! -name SHA256SUMS | sort)
    [ "${#assets[@]}" -gt 0 ] || { echo "release: nothing to checksum in $RELEASE" >&2; exit 2; }

    sum "${assets[@]}" > SHA256SUMS
    cat SHA256SUMS
}

section() {
    awk -v want="$1" '
        /^## / { if (inside) exit; if (index($0, want) == 1) { inside = 1; next } }
        /^\[[^]]+\]: / { if (inside) exit }
        inside { print }
    ' "$ROOT/CHANGELOG.md"
}

notes() {
    local text
    text="$(section "## [$VERSION]")"
    [ -n "$(printf '%s' "$text" | tr -d '[:space:]')" ] || text="$(section '## [Unreleased]')"
    printf '%s\n' "$text"
}

case "${1:-collect}" in
version)   echo "$VERSION" ;;
check-tag) check_tag "${2:-}" ;;
archive)   build_archive ;;
collect)   collect ;;
checksums) checksums ;;
notes)     notes ;;
*)
    echo "usage: env/scripts/release.sh {version|check-tag vX.Y.Z|archive|collect|checksums|notes}" >&2
    exit 2
    ;;
esac
