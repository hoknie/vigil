#!/usr/bin/env bash

set -euo pipefail

REPOSITORY="${VIGIL_REPOSITORY:-hoknie/vigil}"
WANTED="${VIGIL_VERSION:-}"
KIND="${VIGIL_KIND:-}"
TOKEN="${VIGIL_TOKEN:-${GITHUB_TOKEN:-}}"

DRY_RUN=no
REINSTALL=no
KEEP=no
LIST=no

say() { printf '\n\033[1m== %s\033[0m\n' "$*"; }
note() { printf '%s\n' "$*"; }
die() { printf 'vigil install: %s\n' "$*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

usage() {
    cat <<'USAGE'
vigil install — install or update vigil from a GitHub release, as a package

usage: install.sh [options]

  -v, --version VERSION   the release to install, as 1.0.7 or v1.0.7
                          (default: the latest release)
  -k, --kind KIND         deb or rpm (default: whatever this host manages packages with)
  -r, --repository OWNER/NAME
                          where the releases are (default: hoknie/vigil)
  -l, --list              print the releases on offer and leave
  -f, --reinstall         install again even when this version is already here
  -n, --dry-run           say what would be downloaded and installed, and stop
      --keep              leave the downloaded package behind and say where it is
  -h, --help              this text

environment: VIGIL_VERSION, VIGIL_KIND, VIGIL_REPOSITORY, VIGIL_TOKEN (or GITHUB_TOKEN)

Every download is checked against the SHA256SUMS of its own release before anything
is installed. The configuration under /etc/vigil (vigil.yaml, collectors/*.yaml,
watch_fs.yaml) is conffiles: an update keeps the files this host has and never
writes over them.
USAGE
}

while [ $# -gt 0 ]; do
    case "$1" in
    -v|--version) [ $# -ge 2 ] || die "--version wants a version"; WANTED="$2"; shift 2 ;;
    --version=*) WANTED="${1#*=}"; shift ;;
    -k|--kind) [ $# -ge 2 ] || die "--kind wants deb or rpm"; KIND="$2"; shift 2 ;;
    --kind=*) KIND="${1#*=}"; shift ;;
    -r|--repository) [ $# -ge 2 ] || die "--repository wants OWNER/NAME"; REPOSITORY="$2"; shift 2 ;;
    --repository=*) REPOSITORY="${1#*=}"; shift ;;
    -l|--list) LIST=yes; shift ;;
    -f|--reinstall|--force) REINSTALL=yes; shift ;;
    -n|--dry-run) DRY_RUN=yes; shift ;;
    --keep) KEEP=yes; shift ;;
    -h|--help) usage; exit 0 ;;
    *) die "$1 is not an option this script knows; --help lists them" ;;
    esac
done

AUTHORIZED=()

if have curl; then
    [ -z "$TOKEN" ] || AUTHORIZED=(--header "Authorization: Bearer $TOKEN")
    fetch() { curl --fail --silent --show-error --location ${AUTHORIZED[@]+"${AUTHORIZED[@]}"} --output "$2" "$1"; }
    read_out() { curl --fail --silent --show-error --location ${AUTHORIZED[@]+"${AUTHORIZED[@]}"} "$1"; }
    final_url() { curl --silent --show-error --location --head --output /dev/null --write-out '%{url_effective}' "$1"; }
elif have wget; then
    [ -z "$TOKEN" ] || AUTHORIZED=(--header="Authorization: Bearer $TOKEN")
    fetch() { wget --quiet ${AUTHORIZED[@]+"${AUTHORIZED[@]}"} --output-document "$2" "$1"; }
    read_out() { wget --quiet ${AUTHORIZED[@]+"${AUTHORIZED[@]}"} --output-document - "$1"; }
    final_url() { wget --quiet --spider --server-response "$1" 2>&1 | awk '/^  Location: /{ url = $2 } END { print url }'; }
else
    die "neither curl nor wget is installed, and one of them has to fetch the release"
fi

sum_of() {
    if have sha256sum; then sha256sum "$1" | cut -d' ' -f1
    elif have shasum; then shasum -a 256 "$1" | cut -d' ' -f1
    else die "neither sha256sum nor shasum is installed, so the download cannot be checked"; fi
}

case "$(uname -s)" in
Linux) ;;
*) die "vigil watches Linux hosts; this is $(uname -s)" ;;
esac

case "$(uname -m)" in
x86_64|amd64) DEB_ARCH=amd64; RPM_ARCH=x86_64 ;;
aarch64|arm64) DEB_ARCH=arm64; RPM_ARCH=aarch64 ;;
*) die "the releases carry x86_64 and aarch64, and this host is $(uname -m)" ;;
esac

if [ -z "$KIND" ]; then
    if have dpkg; then KIND=deb
    elif have rpm; then KIND=rpm
    else
        die "this host has neither dpkg nor rpm; take the .tar.gz from the release page instead"
    fi
fi

case "$KIND" in
deb) have dpkg || die "--kind deb was asked for and dpkg is not installed" ;;
rpm) have rpm || die "--kind rpm was asked for and rpm is not installed" ;;
*) die "$KIND is neither deb nor rpm" ;;
esac

releases() {
    read_out "https://api.github.com/repos/$REPOSITORY/releases?per_page=30" \
        | awk -F'"' '/"tag_name"/ { print $4 }'
}

latest_version() {
    local tag
    tag="$(read_out "https://api.github.com/repos/$REPOSITORY/releases/latest" 2>/dev/null \
        | awk -F'"' '/"tag_name"/ { print $4; exit }')" || true
    if [ -z "$tag" ]; then
        tag="$(final_url "https://github.com/$REPOSITORY/releases/latest" 2>/dev/null || true)"
        tag="${tag##*/}"
    fi
    case "$tag" in
    ""|*releases*) die "cannot tell what the latest release of $REPOSITORY is; name one with --version" ;;
    esac
    printf '%s\n' "${tag#v}"
}

installed_version() {
    case "$KIND" in
    deb) dpkg-query --show --showformat='${Version}' vigil 2>/dev/null || true ;;
    rpm) rpm --query vigil >/dev/null 2>&1 && rpm --query --queryformat='%{VERSION}' vigil || true ;;
    esac
}

if [ "$LIST" = yes ]; then
    say "the releases of $REPOSITORY"
    releases || die "cannot read the releases of $REPOSITORY"
    exit 0
fi

VERSION="${WANTED#v}"
[ -n "$VERSION" ] || VERSION="$(latest_version)"

case "$KIND" in
deb) ASSET="vigil_${VERSION}.debian.${DEB_ARCH}.deb" ;;
rpm) ASSET="vigil_${VERSION}.el.${RPM_ARCH}.rpm" ;;
esac

BASE="https://github.com/$REPOSITORY/releases/download/v$VERSION"
HERE="$(installed_version)"

say "vigil $VERSION as a $KIND package for $(uname -m)"
note "  from      $BASE/$ASSET"
note "  installed ${HERE:-nothing yet}"

if [ -n "$HERE" ] && [ "$HERE" = "$VERSION" ] && [ "$REINSTALL" = no ]; then
    note ""
    note "vigil $VERSION is already installed. --reinstall installs it again anyway."
    exit 0
fi

if [ "$DRY_RUN" = yes ]; then
    note ""
    note "nothing was downloaded or installed: --dry-run"
    exit 0
fi

if [ "$(id -u)" -ne 0 ]; then
    if have sudo; then
        note ""
        note "installing a package needs root; asking sudo to run this script"
        AGAIN=(--version "$VERSION" --kind "$KIND" --repository "$REPOSITORY")
        [ "$REINSTALL" = no ] || AGAIN+=(--reinstall)
        [ "$KEEP" = no ] || AGAIN+=(--keep)
        exec sudo --preserve-env=VIGIL_TOKEN,GITHUB_TOKEN -- "$0" "${AGAIN[@]}"
    fi
    die "installing a package needs root, and sudo is not here; run this script as root"
fi

WORK="$(mktemp -d)"
cleanup() { [ "$KEEP" = yes ] || rm -rf "$WORK"; }
trap cleanup EXIT

say "downloading"
fetch "$BASE/$ASSET" "$WORK/$ASSET" \
    || die "$ASSET is not in release v$VERSION of $REPOSITORY (--list shows the releases)"
fetch "$BASE/SHA256SUMS" "$WORK/SHA256SUMS" \
    || die "release v$VERSION carries no SHA256SUMS, so the download cannot be checked"

WANT="$(awk -v asset="$ASSET" '{ name = $2; sub(/^\*/, "", name); if (name == asset) { print $1; exit } }' "$WORK/SHA256SUMS")"
[ -n "$WANT" ] || die "SHA256SUMS of v$VERSION says nothing about $ASSET"
GOT="$(sum_of "$WORK/$ASSET")"
[ "$WANT" = "$GOT" ] || die "the download does not match the checksum of the release, and was not installed
  expected $WANT
  got      $GOT"
note "  sha256    $GOT"

defaults() {
    [ -d /etc/vigil ] || return 0
    find /etc/vigil \( -name '*.dpkg-dist' -o -name '*.rpmnew' \) -exec stat -c '%n %i %Y %s' {} + \
        2>/dev/null | sort
}
defaults > "$WORK/before"

say "installing"
case "$KIND" in
deb) dpkg --force-confold --force-confdef --install "$WORK/$ASSET" ;;
rpm) rpm --upgrade --oldpackage --replacepkgs --verbose "$WORK/$ASSET" ;;
esac

NOW="$(installed_version)"
say "installed"
note "  vigil     ${NOW:-unknown}${HERE:+, over $HERE}"
if have vigild; then
    note "  vigild    $(vigild --version 2>/dev/null || echo 'does not say its version')"
fi

if [ -d /run/systemd/system ] && have systemctl; then
    if systemctl is-enabled vigild.service >/dev/null 2>&1; then
        note "  service   $(systemctl is-active vigild.service 2>/dev/null || true), enabled"
    else
        note ""
        note "the daemon is installed and not enabled yet:  systemctl enable --now vigild"
    fi
fi

defaults > "$WORK/after"
comm -13 "$WORK/before" "$WORK/after" | while read -r written _; do
    note ""
    note "${written%.*} is yours and was kept as it is."
    note "the defaults this version ships are beside it, at $written"
done

if [ "$KEEP" = yes ]; then
    note ""
    note "the package is kept at $WORK/$ASSET"
fi
