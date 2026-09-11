#!/usr/bin/env bash

set -euo pipefail

cd "$(dirname "$0")/../.."
ROOT="$PWD"

TARGET="${TARGET:-x86_64-unknown-linux-musl}"
OUT="${OUT:-$ROOT/dist}"

VERSION="${VERSION:-$(awk '/^\[workspace\.package\]/{p=1} p && /^version *= *"/{gsub(/[^0-9A-Za-z.+~-]/,"",$3); print $3; exit}' Cargo.toml)}"
[ -n "$VERSION" ] || { echo "package: cannot read the version out of Cargo.toml" >&2; exit 2; }

case "$TARGET" in
x86_64-*)  DEB_ARCH=amd64; RPM_ARCH=x86_64 ;;
aarch64-*) DEB_ARCH=arm64; RPM_ARCH=aarch64 ;;
*) echo "package: no deb/rpm architecture name known for $TARGET" >&2; exit 2 ;;
esac

BINARIES="$OUT/bin/$TARGET"

say() { printf '\n\033[1m== %s\033[0m\n' "$*"; }

WORK=""
# shellcheck disable=SC2086  # WORK is a space-separated list of mktemp directories
cleanup() { [ -n "$WORK" ] || return 0; rm -rf $WORK; }
trap cleanup EXIT


build_binaries() {
    say "building vigild, vigil and vigil-audit-plugin for $TARGET"

    RUSTFLAGS="${RUSTFLAGS:-} -C linker=rust-lld" \
        cargo build --release --target "$TARGET" \
            --bin vigild --bin vigil --bin vigil-audit-plugin

    mkdir -p "$BINARIES"
    local built="${CARGO_TARGET_DIR:-$ROOT/target}/$TARGET/release"
    install -m 0755 "$built/vigild" "$BINARIES/vigild"
    install -m 0755 "$built/vigil" "$BINARIES/vigil"
    install -m 0755 "$built/vigil-audit-plugin" "$BINARIES/vigil-audit-plugin"

    if command -v file >/dev/null 2>&1; then
        file "$BINARIES/vigild"
        file "$BINARIES/vigild" | grep -q 'statically linked\|static-pie' \
            || { echo "package: $BINARIES/vigild is not statically linked" >&2; exit 1; }
    fi
    ls -l "$BINARIES"
}


stage() {
    local tree="$1"
    rm -rf "$tree"

    [ -x "$BINARIES/vigild" ] || {
        echo "package: $BINARIES/vigild is missing — run 'env/scripts/package.sh binaries' first" >&2
        exit 2
    }

    install -D -m 0755 "$BINARIES/vigild" "$tree/usr/sbin/vigild"
    install -D -m 0755 "$BINARIES/vigil" "$tree/usr/bin/vigil"
    install -D -m 0755 "$BINARIES/vigil-audit-plugin" "$tree/usr/sbin/vigil-audit-plugin"

    install -d -m 0700 "$tree/etc/vigil"
    install -m 0600 "$ROOT/config/vigil.example.yaml" "$tree/etc/vigil/vigil.yaml"

    install -D -m 0644 "$ROOT/packaging/systemd/vigild.service" \
        "$tree/usr/lib/systemd/system/vigild.service"
    install -D -m 0644 "$ROOT/packaging/systemd/vigil-firewall.service" \
        "$tree/usr/lib/systemd/system/vigil-firewall.service"
    install -D -m 0644 "$ROOT/packaging/systemd/vigil-firewall.timer" \
        "$tree/usr/lib/systemd/system/vigil-firewall.timer"
    install -D -m 0644 "$ROOT/packaging/systemd/vigil-tmpfiles.conf" \
        "$tree/usr/lib/tmpfiles.d/vigil.conf"
    install -D -m 0644 "$ROOT/packaging/logrotate/vigil" "$tree/etc/logrotate.d/vigil"

    install -d -m 0750 "$tree/etc/audit"
    install -d -m 0750 "$tree/etc/audit/rules.d"
    install -d -m 0750 "$tree/etc/audit/plugins.d"
    install -m 0640 "$ROOT/packaging/audit/vigil-exec.rules" \
        "$tree/etc/audit/rules.d/vigil-exec.rules"
    install -m 0640 "$ROOT/packaging/audit/vigil.conf" "$tree/etc/audit/plugins.d/vigil.conf"

    install -d -m 0700 "$tree/var/lib/vigil"
    install -d -m 0700 "$tree/var/lib/vigil/firewall"
    install -d -m 0700 "$tree/var/log/vigil"

    install -d -m 0755 "$tree/usr/share/doc/vigil"
    install -m 0644 "$ROOT/README.md" "$tree/usr/share/doc/vigil/README.md"
    install -m 0644 "$ROOT/LICENSE" "$tree/usr/share/doc/vigil/copyright"
    install -m 0644 "$ROOT/NOTICE" "$tree/usr/share/doc/vigil/NOTICE"
    install -m 0644 "$ROOT/THIRD-PARTY.md" "$tree/usr/share/doc/vigil/THIRD-PARTY.md"
}

build_deb() {
    command -v dpkg-deb >/dev/null || { echo "package: no dpkg-deb here" >&2; exit 2; }
    local tree; tree="$(mktemp -d)"; WORK="$WORK $tree"

    say "staging the install tree"
    stage "$tree"

    install -d -m 0755 "$tree/DEBIAN"
    local size; size="$(du -k -s "$tree" | cut -f1)"
    sed -e "s/@VERSION@/$VERSION/" -e "s/@ARCH@/$DEB_ARCH/" -e "s/@INSTALLED_SIZE@/$size/" \
        "$ROOT/packaging/deb/control.in" > "$tree/DEBIAN/control"

    install -m 0644 "$ROOT/packaging/deb/conffiles" "$tree/DEBIAN/conffiles"
    for script in postinst prerm postrm; do
        install -m 0755 "$ROOT/packaging/deb/$script" "$tree/DEBIAN/$script"
    done

    ( cd "$tree" && find . -path ./DEBIAN -prune -o -type f -print0 \
        | xargs -0 md5sum | sed 's| \./| |' > DEBIAN/md5sums ) || true
    chmod 0644 "$tree/DEBIAN/md5sums"

    mkdir -p "$OUT"
    local deb="$OUT/vigil_${VERSION}_${DEB_ARCH}.deb"
    say "dpkg-deb"
    dpkg-deb --root-owner-group --build "$tree" "$deb"
    dpkg-deb --info "$deb"
    dpkg-deb --contents "$deb"
    echo "$deb"
}

build_rpm() {
    command -v rpmbuild >/dev/null || { echo "package: no rpmbuild" >&2; exit 2; }
    local tree; tree="$(mktemp -d)"
    local build; build="$(mktemp -d)"
    WORK="$WORK $tree $build"

    say "staging the install tree"
    stage "$tree"

    mkdir -p "$OUT"
    say "rpmbuild"
    rpmbuild -bb "$ROOT/packaging/rpm/vigil.spec" \
        --target "$RPM_ARCH" \
        --define "vigil_version $VERSION" \
        --define "_sourcedir $tree" \
        --define "_topdir $build" \
        --define "_rpmdir $OUT" \
        --define "_build_id_links none" \
        --define "dist %{nil}"

    local wanted="vigil-${VERSION}-1.${RPM_ARCH}.rpm"
    local built; built="$(find "$OUT/$RPM_ARCH" -name "$wanted" -print -quit 2>/dev/null || true)"
    if [ -n "$built" ]; then
        mv -f "$built" "$OUT/"
        rmdir "$OUT/$RPM_ARCH" 2>/dev/null || true
    fi

    [ -f "$OUT/$wanted" ] || { echo "package: rpmbuild produced nothing findable" >&2; exit 1; }
    rpm -qpi "$OUT/$wanted"
    rpm -qpl "$OUT/$wanted"
    echo "$OUT/$wanted"
}

case "${1:-all}" in
binaries) build_binaries ;;
stage)    stage "${2:-$OUT/stage}"; find "${2:-$OUT/stage}" -printf '%M %p\n' | sort ;;
deb)      build_deb ;;
rpm)      build_rpm ;;
*)
    echo "usage: env/scripts/package.sh {binaries|stage|deb|rpm}" >&2
    exit 2
    ;;
esac
