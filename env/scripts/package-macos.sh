#!/usr/bin/env bash

set -euo pipefail

cd "$(dirname "$0")/../.."
ROOT="$PWD"

OUT="${OUT:-$ROOT/dist}"
TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)
UNIVERSAL="universal-apple-darwin"
BINARIES="$OUT/bin/$UNIVERSAL"
IDENTIFIER="vigil.agent"
PROGRAMS=(vigild vigil vigil-container-dump vigil-firewall-dump vigil-launches-spool)

VERSION="${VERSION:-$(awk '/^\[workspace\.package\]/{p=1} p && /^version *= *"/{gsub(/[^0-9A-Za-z.+~-]/,"",$3); print $3; exit}' Cargo.toml)}"
[ -n "$VERSION" ] || { echo "package-macos: cannot read the version out of Cargo.toml" >&2; exit 2; }

NAME="vigil_${VERSION}.macos.universal"
PKG="$OUT/$NAME.pkg"
ARCHIVE="$OUT/$NAME.tar.gz"

LINUX_ETC="/etc/vigil"
LINUX_STATE="/var/lib/vigil"
LINUX_LOGS="/var/log/vigil"
LINUX_RUN="/run/vigil"
MACOS_ETC="/usr/local/etc/vigil"
MACOS_STATE="/usr/local/var/lib/vigil"
MACOS_LOGS="/usr/local/var/log/vigil"
MACOS_RUN="/var/run/vigil"

say() { printf '\n\033[1m== %s\033[0m\n' "$*"; }
die() { printf 'package-macos: %s\n' "$*" >&2; exit 1; }

WORK=""
# shellcheck disable=SC2086
cleanup() { [ -n "$WORK" ] || return 0; rm -rf $WORK; }
trap cleanup EXIT

[ "$(uname -s)" = Darwin ] || die "a .pkg is built with pkgbuild and productbuild, which only macOS has"

build_binaries() {
    local installed
    installed="$(rustup target list --installed 2>/dev/null || true)"
    for target in "${TARGETS[@]}"; do
        grep -qx "$target" <<<"$installed" \
            || die "the $target standard library is not installed: rustup target add $target"
    done

    local arguments=()
    for program in "${PROGRAMS[@]}"; do arguments+=(--bin "$program"); done

    for target in "${TARGETS[@]}"; do
        say "building ${PROGRAMS[*]} for $target"
        MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-11.0}" \
            cargo build --release --locked --target "$target" "${arguments[@]}"
    done

    say "gluing the two into universal binaries"
    mkdir -p "$BINARIES"
    local built="${CARGO_TARGET_DIR:-$ROOT/target}"
    for program in "${PROGRAMS[@]}"; do
        lipo -create \
            "$built/aarch64-apple-darwin/release/$program" \
            "$built/x86_64-apple-darwin/release/$program" \
            -output "$BINARIES/$program"
        chmod 0755 "$BINARIES/$program"
        lipo "$BINARIES/$program" -verify_arch arm64 x86_64 \
            || die "$BINARIES/$program does not hold both arm64 and x86_64"
        lipo -info "$BINARIES/$program"
    done
}

moved() {
    local ends="([/ \"']|\$)"
    local begins="(^|[^/A-Za-z0-9._-])"
    sed -E \
        -e "s#${begins}${LINUX_STATE}${ends}#\\1${MACOS_STATE}\\2#g" \
        -e "s#${begins}${LINUX_LOGS}${ends}#\\1${MACOS_LOGS}\\2#g" \
        -e "s#${begins}${LINUX_RUN}${ends}#\\1${MACOS_RUN}\\2#g" \
        -e "s#${begins}${LINUX_ETC}${ends}#\\1${MACOS_ETC}\\2#g" \
        "$1"
}

defaults() {
    local into="$1"
    install -d -m 0755 "$into" "$into/collectors"
    moved "$ROOT/config/vigil.example.yaml" > "$into/vigil.yaml"
    for collectors in "$ROOT"/config/collectors/*.yaml; do
        moved "$collectors" > "$into/collectors/$(basename "$collectors")"
    done
    moved "$ROOT/config/watch_fs.yaml" > "$into/watch_fs.yaml"
    find "$into" -type f -exec chmod 0644 {} +
    if grep -rEn "(^|[^/A-Za-z0-9._-])($LINUX_ETC|$LINUX_STATE|$LINUX_LOGS|$LINUX_RUN)([/ \"']|\$)" "$into"; then
        die "a Linux path is left in the configuration a Mac would be given"
    fi
}

stage() {
    local tree="$1"
    rm -rf "$tree"

    for program in "${PROGRAMS[@]}"; do
        [ -x "$BINARIES/$program" ] \
            || die "$BINARIES/$program is missing — run 'env/scripts/package-macos.sh binaries' first"
    done

    install -d -m 0755 "$tree/usr/local/sbin" "$tree/usr/local/bin" "$tree/usr/local/libexec/vigil"
    install -m 0755 "$BINARIES/vigild" "$tree/usr/local/sbin/vigild"
    install -m 0755 "$BINARIES/vigil" "$tree/usr/local/bin/vigil"
    for helper in vigil-container-dump vigil-firewall-dump vigil-launches-spool; do
        install -m 0755 "$BINARIES/$helper" "$tree/usr/local/libexec/vigil/$helper"
    done
    install -m 0755 "$ROOT/packaging/macos/uninstall" "$tree/usr/local/libexec/vigil/uninstall"

    install -d -m 0755 "$tree/Library/LaunchDaemons"
    for plist in "$ROOT"/packaging/macos/launchd/*.plist; do
        plutil -lint "$plist" >/dev/null || die "$plist is not a property list launchd can read"
        install -m 0644 "$plist" "$tree/Library/LaunchDaemons/$(basename "$plist")"
    done

    install -d -m 0755 "$tree/usr/local/share/vigil"
    defaults "$tree/usr/local/share/vigil/defaults"

    install -d -m 0755 "$tree/usr/local/share/doc/vigil"
    install -m 0644 "$ROOT/README.md" "$tree/usr/local/share/doc/vigil/README.md"
    install -m 0644 "$ROOT/packaging/macos/README.md" "$tree/usr/local/share/doc/vigil/README-macos.md"
    install -m 0644 "$ROOT/LICENSE" "$tree/usr/local/share/doc/vigil/LICENSE"
    install -m 0644 "$ROOT/NOTICE" "$tree/usr/local/share/doc/vigil/NOTICE"
    install -m 0644 "$ROOT/THIRD-PARTY.md" "$tree/usr/local/share/doc/vigil/THIRD-PARTY.md"
}

build_pkg() {
    local work; work="$(mktemp -d)"; WORK="$WORK $work"
    local tree="$work/root"
    local scripts="$work/scripts"

    say "staging the install tree"
    stage "$tree"

    install -d -m 0755 "$scripts"
    install -m 0755 "$ROOT/packaging/macos/scripts/preinstall" "$scripts/preinstall"
    install -m 0755 "$ROOT/packaging/macos/scripts/postinstall" "$scripts/postinstall"

    say "pkgbuild"
    pkgbuild \
        --root "$tree" \
        --identifier "$IDENTIFIER" \
        --version "$VERSION" \
        --scripts "$scripts" \
        --ownership recommended \
        --install-location / \
        "$work/vigil-component.pkg"

    sed -e "s/@VERSION@/$VERSION/g" -e "s/@IDENTIFIER@/$IDENTIFIER/g" \
        "$ROOT/packaging/macos/distribution.xml" > "$work/distribution.xml"

    say "productbuild"
    mkdir -p "$OUT"
    rm -f "$PKG"
    productbuild \
        --distribution "$work/distribution.xml" \
        --package-path "$work" \
        "$PKG"
    pkgutil --check-signature "$PKG" || true
    echo "$PKG"
}

build_archive() {
    local work; work="$(mktemp -d)"; WORK="$WORK $work"
    local tree="$work/$NAME"

    say "the archive"
    install -d -m 0755 "$tree" "$tree/launchd"
    for program in "${PROGRAMS[@]}"; do
        [ -x "$BINARIES/$program" ] \
            || die "$BINARIES/$program is missing — run 'env/scripts/package-macos.sh binaries' first"
        install -m 0755 "$BINARIES/$program" "$tree/$program"
    done
    install -m 0755 "$ROOT/packaging/macos/uninstall" "$tree/uninstall"
    for plist in "$ROOT"/packaging/macos/launchd/*.plist; do
        install -m 0644 "$plist" "$tree/launchd/$(basename "$plist")"
    done
    defaults "$tree/config"
    install -m 0644 "$ROOT/packaging/macos/README.md" "$tree/README-macos.md"
    install -m 0644 "$ROOT/README.md" "$tree/README.md"
    install -m 0644 "$ROOT/LICENSE" "$tree/LICENSE"
    install -m 0644 "$ROOT/NOTICE" "$tree/NOTICE"
    install -m 0644 "$ROOT/THIRD-PARTY.md" "$tree/THIRD-PARTY.md"

    mkdir -p "$OUT"
    rm -f "$ARCHIVE"
    COPYFILE_DISABLE=1 tar -C "$work" --uid 0 --gid 0 --uname root --gname wheel -cf - "$NAME" \
        | gzip -9 -n > "$ARCHIVE"
    tar -tzf "$ARCHIVE"
    echo "$ARCHIVE"
}

check_pkg() {
    [ -f "$PKG" ] || die "$PKG is missing — run 'env/scripts/package-macos.sh pkg' first"
    local work; work="$(mktemp -d)"; WORK="$WORK $work"

    say "what $PKG holds"
    pkgutil --expand "$PKG" "$work/expanded"
    local component="$work/expanded/vigil-component.pkg"
    [ -d "$component" ] || die "the product holds no vigil-component.pkg"

    mkdir -p "$work/payload"
    (cd "$work/payload" && gzip -dc "$component/Payload" | cpio -i --quiet)

    local listing="$work/listing"
    lsbom -p MUGf "$component/Bom" > "$work/bom"
    awk -F'\t' '$4 !~ /\/\._/ { sub(/ +$/, "", $1); printf "%s %s:%s %s\n", $1, $2, $3, $4 }' "$work/bom" > "$listing"
    cat "$listing"
    if grep -q '/\._' "$work/bom"; then
        echo
        echo "  note  the payload also carries the extended attributes of the files it was built"
        echo "        from, as AppleDouble entries; installer puts them back as attributes, not as files"
    fi

    local failed=0
    expect() {
        if grep -q -- "$1" "$listing"; then
            printf '  ok    %s\n' "$2"
        else
            printf '  FAIL  %s\n' "$2"; failed=1
        fi
    }
    owned() {
        if awk -F'\t' -v path="./$1" -v want="$2" '$4 == path && $2 "/" $3 == want { found = 1 } END { exit !found }' "$work/bom"; then
            printf '  ok    %s is %s\n' "$1" "$2"
        else
            printf '  FAIL  %s is not %s\n' "$1" "$2"; failed=1
        fi
    }
    moded() {
        if awk -F'\t' -v path="./$1" -v want="$2" '{ sub(/ +$/, "", $1) } $4 == path && $1 == want { found = 1 } END { exit !found }' "$work/bom"; then
            printf '  ok    %s is %s\n' "$1" "$2"
        else
            printf '  FAIL  %s is not %s\n' "$1" "$2"; failed=1
        fi
    }

    say "the checks"
    for program in usr/local/sbin/vigild usr/local/bin/vigil usr/local/libexec/vigil/vigil-container-dump \
        usr/local/libexec/vigil/vigil-firewall-dump usr/local/libexec/vigil/vigil-launches-spool; do
        moded "$program" "-rwxr-xr-x"
        owned "$program" "root/wheel"
        lipo "$work/payload/$program" -verify_arch arm64 x86_64 \
            && printf '  ok    %s holds arm64 and x86_64\n' "$program" \
            || { printf '  FAIL  %s is not universal\n' "$program"; failed=1; }
    done
    moded usr/local/libexec/vigil/uninstall "-rwxr-xr-x"
    for label in vigil.vigild vigil.firewall vigil.containers vigil.launches; do
        moded "Library/LaunchDaemons/$label.plist" "-rw-r--r--"
        owned "Library/LaunchDaemons/$label.plist" "root/wheel"
    done
    expect ' ./usr/local/share/vigil/defaults/vigil.yaml$' "the defaults are shipped beside the configuration, never over it"
    expect ' ./usr/local/share/vigil/defaults/watch_fs.yaml$' "the watch list is among them"
    if [ -e "$work/payload/usr/local/etc/vigil" ]; then
        printf '  FAIL  the payload writes into /usr/local/etc/vigil, over what the host has\n'; failed=1
    else
        printf '  ok    nothing in the payload is written into /usr/local/etc/vigil\n'
    fi
    grep -q "state_dir: $MACOS_STATE\$" "$work/payload/usr/local/share/vigil/defaults/vigil.yaml" \
        && printf '  ok    the shipped state_dir is %s\n' "$MACOS_STATE" \
        || { printf '  FAIL  the shipped state_dir is not %s\n' "$MACOS_STATE"; failed=1; }
    grep -q "socket_path: $MACOS_RUN/vigil.sock\$" "$work/payload/usr/local/share/vigil/defaults/vigil.yaml" \
        && printf '  ok    the shipped socket_path is under %s\n' "$MACOS_RUN" \
        || { printf '  FAIL  the shipped socket_path is not under %s\n' "$MACOS_RUN"; failed=1; }
    for script in preinstall postinstall; do
        [ -x "$component/Scripts/$script" ] \
            && printf '  ok    the %s script is there and runs\n' "$script" \
            || { printf '  FAIL  the %s script is missing\n' "$script"; failed=1; }
        sh -n "$component/Scripts/$script" \
            && printf '  ok    the %s script parses\n' "$script" \
            || { printf '  FAIL  the %s script does not parse\n' "$script"; failed=1; }
    done
    "$work/payload/usr/local/sbin/vigild" configure --dry-run "$MACOS_ETC/vigil.yaml" 2>/dev/null \
        | awk -v want="==> $MACOS_ETC/vigil.yaml <==" '/^==> .* <==$/ { inside = ($0 == want); next } inside' \
        > "$work/configured.yaml"
    if diff -u <(cat "$work/payload/usr/local/share/vigil/defaults/vigil.yaml"; echo) "$work/configured.yaml"; then
        printf '  ok    the shipped vigil.yaml is the one vigild configure writes on a Mac\n'
    else
        printf '  FAIL  the shipped vigil.yaml and vigild configure disagree about a path\n'; failed=1
    fi
    "$work/payload/usr/local/sbin/vigild" --version \
        && printf '  ok    the vigild of the payload runs here\n' \
        || { printf '  FAIL  the vigild of the payload does not run here\n'; failed=1; }

    [ "$failed" -eq 0 ] || die "the package is not what it should be"
    say "the package is what it should be"
}

check_scripts() {
    local work; work="$(mktemp -d)"; WORK="$WORK $work"
    local volume="$work/volume"
    local etc="$volume$MACOS_ETC"
    local shipped="$volume/usr/local/share/vigil/defaults"
    local failed=0

    say "the install scripts, run against a scratch volume as this user"
    for script in preinstall postinstall; do
        sed -e 's/chown root:wheel "$1"/: chown/' -e 's/ -o root -g wheel//' \
            "$ROOT/packaging/macos/scripts/$script" > "$work/$script"
    done
    installed() {
        sh "$work/preinstall" vigil.pkg / "$volume" >/dev/null
        [ -z "${1:-}" ] || "$1"
        sh "$work/postinstall" vigil.pkg / "$volume"
    }
    verdict() {
        if eval "$1"; then printf '  ok    %s\n' "$2"; else printf '  FAIL  %s\n' "$2"; failed=1; fi
    }

    defaults "$shipped" >/dev/null
    installed
    for directory in "$etc" "$etc/collectors" "$etc/suppressions" "$etc/reporters" \
        "$volume$MACOS_STATE" "$volume$MACOS_STATE/firewall" "$volume$MACOS_STATE/containers" \
        "$volume$MACOS_STATE/launches" "$volume$MACOS_LOGS" "$volume$MACOS_RUN"; do
        verdict "[ \"\$(stat -f %Sp '$directory')\" = drwx------ ]" "${directory#"$volume"} is made 0700"
    done
    verdict "[ \"\$(stat -f %Sp '$etc/vigil.yaml')\" = -rw------- ]" "a configuration file is put in place 0600"
    verdict "cmp -s '$shipped/collectors/users.yaml' '$etc/collectors/users.yaml'" "a fresh install gets the shipped defaults"

    echo "retention_days: 30" >> "$etc/vigil.yaml"
    new_defaults() {
        echo "# a default of the next version" >> "$shipped/vigil.yaml"
        echo "# a default of the next version" >> "$shipped/watch_fs.yaml"
    }
    installed new_defaults
    verdict "tail -1 '$etc/vigil.yaml' | grep -q 'retention_days: 30'" "an upgrade keeps a file the operator edited"
    verdict "cmp -s '$shipped/vigil.yaml' '$etc/vigil.yaml.new'" "and writes the new defaults beside it as .new"
    verdict "cmp -s '$shipped/watch_fs.yaml' '$etc/watch_fs.yaml'" "a file nobody edited is brought to the new defaults"
    verdict "[ ! -e '$etc/watch_fs.yaml.new' ]" "and gets no .new"
    verdict "[ ! -e '$volume/usr/local/share/vigil/defaults.before' ]" "the copy of the previous defaults is gone afterwards"

    [ "$failed" -eq 0 ] || die "the install scripts do not do what they should"
    say "the install scripts do what they should"
}

case "${1:-all}" in
binaries) build_binaries ;;
stage)    stage "${2:-$OUT/stage-macos}"; find "${2:-$OUT/stage-macos}" -exec stat -f '%Sp %N' {} + | sort -k2 ;;
pkg)      build_pkg ;;
archive)  build_archive ;;
check)    check_pkg; check_scripts ;;
scripts)  check_scripts ;;
all)      build_binaries; build_pkg; build_archive; check_pkg; check_scripts ;;
*)
    echo "usage: env/scripts/package-macos.sh {all|binaries|stage|pkg|archive|check|scripts}" >&2
    exit 2
    ;;
esac
