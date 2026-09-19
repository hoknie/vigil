#!/bin/sh
set -eu

cd "$(dirname "$0")/../.."
ROOT="$PWD"
OUT="${OUT:-target/distributions}"
ROUND_SECONDS="${ROUND_SECONDS:-8}"
DROP_IMAGES="${DROP_IMAGES:-0}"
IMAGES="debian:12 ubuntu:22.04 ubuntu:24.04 almalinux:8 almalinux:9 rockylinux:9 fedora:latest alpine:3.20 archlinux:latest opensuse/leap:15.6 opensuse/tumbleweed"
AMD64_ONLY="archlinux:latest"
COMPOSE="${COMPOSE:-docker compose -f env/docker/compose.yaml}"

say() { printf '\n\033[1m== %s\033[0m\n' "$*"; }

target_of() {
    case "$1" in
    arm64 | aarch64) echo aarch64-unknown-linux-musl ;;
    x86_64 | amd64) echo x86_64-unknown-linux-musl ;;
    *) echo "distributions: no musl target known for $1" >&2; exit 2 ;;
    esac
}

platform_of() {
    case "$1" in
    aarch64-*) echo linux/arm64 ;;
    x86_64-*) echo linux/amd64 ;;
    esac
}

native_target() { target_of "$(uname -m)"; }

build() {
    native="$(native_target)"
    platform="$(platform_of "$native")"
    for target in $native x86_64-unknown-linux-musl; do
        say "static binaries for $target"
        $COMPOSE run --rm -T -e TARGET="$target" -e OUT="/work/$OUT" vigil ./env/scripts/package.sh binaries
    done
    say "the .deb for $native"
    docker run --rm --platform "$platform" -v "$ROOT":/work -w /work -e TARGET="$native" -e OUT="/work/$OUT" \
        debian:12 timeout -s KILL 300 ./env/scripts/package.sh deb
    say "the .rpm for $native"
    docker run --rm --platform "$platform" -v "$ROOT":/work -w /work -e TARGET="$native" -e OUT="/work/$OUT" \
        almalinux:9 bash -c 'timeout -s KILL 600 sh -c "dnf -y -q install rpm-build >/dev/null && ./env/scripts/package.sh rpm"'
}

slug_of() { echo "$1" | tr '/:' '--'; }

run() {
    # shellcheck disable=SC2086
    [ "$#" -gt 0 ] || set -- $IMAGES
    native="$(native_target)"
    mkdir -p "$OUT/matrix"
    matrix="$OUT/matrix/matrix.tsv"
    : > "$matrix"

    for image in "$@"; do
        target="$native"
        case " $AMD64_ONLY " in
        *" $image "*) target=x86_64-unknown-linux-musl ;;
        esac
        platform="$(platform_of "$target")"
        log="$OUT/matrix/$(slug_of "$image").log"

        say "$image ($platform)"
        had=0
        docker image inspect "$image" >/dev/null 2>&1 && had=1
        if [ ! -x "$OUT/bin/$target/vigild" ]; then
            echo "skipped: $OUT/bin/$target/vigild is not built (env/scripts/distributions.sh build)"
            printf '%s\t-\tskipped\tno %s build\n' "$image" "$target" >> "$matrix"
            continue
        fi
        if ! docker run --rm --platform "$platform" "$image" true >/dev/null 2>&1; then
            echo "skipped: $image does not start as $platform here (no image for it, or no emulation)"
            printf '%s\t-\tskipped\t%s does not start here\n' "$image" "$platform" >> "$matrix"
            continue
        fi

        if docker run --rm --platform "$platform" --cap-add NET_ADMIN --cap-add SYS_PTRACE \
            -v "$ROOT":/work:ro -v "$ROOT/$OUT":/dist:ro -e TARGET="$target" -e ROUND_SECONDS="$ROUND_SECONDS" \
            "$image" sh /work/env/scripts/distributions.sh inside > "$log" 2>&1; then
            echo "done: $log"
        else
            echo "FAILED: $log"
            tail -20 "$log"
        fi
        if [ "$DROP_IMAGES" = 1 ] && [ "$had" = 0 ]; then
            docker rmi "$image" >/dev/null 2>&1 || true
        fi
        sed -n "s|^MATRIX	|$image	|p" "$log" >> "$matrix"
        sed -n 's/^MATRIX	\([^	]*\)	\([^	]*\).*/  \1: \2/p' "$log"
    done

    say "distribution × collector"
    awk -F '\t' '
        { image[$1] = 1; if ($2 != "-") name[$2] = 1; cell[$1 SUBSEP $2] = $3; order[++n] = $1 }
        END {
            printf "%-22s", "";
            columns = 0;
            for (c in name) listed[++columns] = c;
            for (i = 1; i <= columns; i++) for (j = i + 1; j <= columns; j++)
                if (listed[j] < listed[i]) { t = listed[i]; listed[i] = listed[j]; listed[j] = t }
            for (i = 1; i <= columns; i++) printf " %-12.12s", listed[i];
            printf "\n";
            for (k = 1; k <= n; k++) {
                row = order[k];
                if (row in shown) continue;
                shown[row] = 1;
                printf "%-22.22s", row;
                if ((row SUBSEP "-") in cell) { printf " %s\n", cell[row SUBSEP "-"]; continue }
                for (i = 1; i <= columns; i++) {
                    state = cell[row SUBSEP listed[i]];
                    printf " %-12.12s", (state == "" ? "." : state);
                }
                printf "\n";
            }
        }' "$matrix"
    echo
    echo "each reason: $matrix; each run in full: $OUT/matrix/<image>.log"
}

family_of() {
    for id in "$@"; do
        case "$id" in
        debian | ubuntu) echo debian; return ;;
        rhel | centos | fedora | almalinux | rocky) echo rhel; return ;;
        alpine) echo alpine; return ;;
        arch | archlinux) echo arch; return ;;
        suse | opensuse | opensuse-leap | opensuse-tumbleweed | sles) echo suse; return ;;
        esac
    done
    echo unknown
}

prepare() {
    case "$1" in
    debian)
        export DEBIAN_FRONTEND=noninteractive
        apt-get -qq update >/dev/null
        apt-get -qq install -y --no-install-recommends sudo cron nftables openssh-server iproute2 procps passwd >/dev/null
        ;;
    rhel) dnf -y -q install sudo cronie nftables openssh-server iproute procps-ng shadow-utils >/dev/null ;;
    alpine) apk add -q sudo nftables openssh-server shadow iproute2 procps >/dev/null ;;
    arch)
        for option in DisableSandbox DisableSandboxFilesystem DisableSandboxSyscalls; do
            grep -q "^$option" /etc/pacman.conf || sed -i "s/^\\[options\\]/[options]\\n$option/" /etc/pacman.conf
        done
        pacman -Sy --noconfirm --needed -q sudo cronie nftables openssh iproute2 procps-ng >/dev/null
        ;;
    suse) zypper -n -q in sudo cronie nftables openssh-server iproute2 procps shadow >/dev/null ;;
    esac
    if command -v systemd-tmpfiles >/dev/null 2>&1; then
        systemd-tmpfiles --create >/dev/null 2>&1 || true
    fi
}

install_vigil() {
    family="$1"
    case "$TARGET" in
    aarch64-*) deb_arch=arm64; rpm_arch=aarch64 ;;
    *) deb_arch=amd64; rpm_arch=x86_64 ;;
    esac
    deb=""
    rpm=""
    for candidate in /dist/vigil_*_"$deb_arch".deb; do if [ -f "$candidate" ]; then deb="$candidate"; fi; done
    for candidate in /dist/vigil-*."$rpm_arch".rpm; do if [ -f "$candidate" ]; then rpm="$candidate"; fi; done

    case "$family" in
    debian)
        if [ -n "$deb" ]; then
            echo "installing $deb"
            dpkg -i "$deb"
            return
        fi
        ;;
    rhel | suse)
        if [ -n "$rpm" ]; then
            echo "installing $rpm"
            rpm -i "$rpm"
            return
        fi
        ;;
    esac

    echo "no package for this family: the four binaries of $TARGET are copied in place"
    install -D -m 0755 "/dist/bin/$TARGET/vigild" /usr/sbin/vigild
    install -D -m 0755 "/dist/bin/$TARGET/vigil" /usr/bin/vigil
    install -D -m 0755 "/dist/bin/$TARGET/vigil-audit-plugin" /usr/sbin/vigil-audit-plugin
    install -D -m 0755 "/dist/bin/$TARGET/vigil-container-dump" /usr/sbin/vigil-container-dump
}

privileged_group() {
    if getent group sudo >/dev/null 2>&1; then
        echo sudo
        return
    fi
    getent group wheel >/dev/null 2>&1 || groupadd -r wheel
    echo wheel
}

shape_the_host() {
    family="$1"
    group="$(privileged_group)"

    useradd -m -s /bin/sh -G "$group" alice
    mkdir -p /home/alice/.ssh
    echo "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHv7kX0x3tqD0dG3wYq7m1q2c4VxLz8m9kqvYQ2n5p1T alice@laptop" \
        > /home/alice/.ssh/authorized_keys
    chown -R alice /home/alice/.ssh
    chmod 0700 /home/alice/.ssh
    chmod 0600 /home/alice/.ssh/authorized_keys
    echo "alice: an account in $group with a key"

    mkdir -p /etc/sudoers.d
    echo 'alice ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/90-alice
    chmod 0440 /etc/sudoers.d/90-alice
    if [ -d /usr/etc/sudoers.d ]; then
        echo 'alice ALL=(ALL) NOPASSWD: /usr/bin/id' > /usr/etc/sudoers.d/80-vendor
        chmod 0440 /usr/etc/sudoers.d/80-vendor
    fi

    if printf '*/5 * * * * /usr/local/bin/check --quiet\n' | crontab -u alice - 2>/tmp/crontab.said; then
        echo "crontab -u alice: $(crontab -l -u alice 2>/dev/null | head -1)"
    else
        echo "crontab -u alice refused: $(head -1 /tmp/crontab.said)"
        for spool in /var/spool/cron/tabs /var/spool/cron/crontabs /var/spool/cron; do
            [ -d "$spool" ] || continue
            printf '*/5 * * * * /usr/local/bin/check --quiet\n' > "$spool/alice"
            chmod 0600 "$spool/alice"
            echo "the crontab of alice is written into $spool by hand"
            break
        done
    fi
    mkdir -p /etc/cron.d
    echo '*/10 * * * * root /usr/local/bin/rotate' > /etc/cron.d/vigil-check
    case "$family" in
    rhel | arch | suse) echo '17 3 * * * root /usr/local/bin/backup' > /etc/cron.d/backup.job ;;
    alpine)
        printf '#!/bin/sh\n/usr/local/bin/sweep\n' > /etc/periodic/daily/sweep
        chmod 0755 /etc/periodic/daily/sweep
        mkdir -p /etc/local.d
        printf '#!/bin/sh\n/usr/local/bin/beacon &\n' > /etc/local.d/beacon.start
        chmod 0755 /etc/local.d/beacon.start
        ;;
    esac

    ssh-keygen -A >/dev/null 2>&1 || true
    mkdir -p /run/sshd
    sshd="$(command -v sshd || echo /usr/sbin/sshd)"
    if "$sshd" 2>/dev/null; then echo "sshd listening"; else echo "sshd did not start"; fi

    nft="$(command -v nft || true)"
    if [ -n "$nft" ] && "$nft" add table inet vigil_check 2>/dev/null; then
        "$nft" add chain inet vigil_check input '{ type filter hook input priority 0; policy accept; }'
        "$nft" add rule inet vigil_check input tcp dport 22 accept
        echo "nft: a table with one rule ($($nft --version 2>/dev/null))"
    else
        echo "nft: no table could be added here"
    fi
}

dump_like_the_timers() {
    mkdir -p /var/lib/vigil/firewall /var/lib/vigil/containers /run/vigil /var/log/vigil
    chmod 0700 /var/lib/vigil /var/lib/vigil/firewall /var/lib/vigil/containers /run/vigil
    nft="$(command -v nft || true)"
    if [ -n "$nft" ]; then
        "$nft" --json list ruleset > /var/lib/vigil/firewall/ruleset.json 2>/dev/null \
            || echo "nft --json list ruleset refused"
    fi
    /usr/sbin/vigil-container-dump --directory /var/lib/vigil/containers >/dev/null 2>&1 \
        || echo "vigil-container-dump refused"
}

seen() {
    if grep -rqF -- "$2" /var/lib/vigil/baselines 2>/dev/null; then
        echo "  seen      $1"
        printf 'MATRIX\t%s\tok\tseen: %s\n' "$3" "$1" >> /tmp/expected
    else
        echo "  NOT SEEN  $1"
        printf 'MATRIX\t%s\tmissed\tnot seen: %s\n' "$3" "$1" >> /tmp/expected
    fi
}

expected() {
    family="$1"
    : > /tmp/expected
    seen "alice in the sudoers.d grant" "90-alice" sudoers.d
    seen "the crontab of alice" "/usr/local/bin/check" crontab
    seen "a line of /etc/cron.d" "/usr/local/bin/rotate" cron.d
    seen "sshd listening on 22" '"port":22' listening
    seen "the rule of the nft table" "vigil_check" nft
    if [ -f /usr/etc/sudoers ] && [ ! -f /etc/sudoers ]; then
        seen "a grant of /usr/etc/sudoers, the only sudoers here" '/usr/etc/sudoers"' sudoers-usr-etc
        seen "a grant in /usr/etc/sudoers.d" "80-vendor" sudoers.d-usr-etc
    fi
    case "$family" in
    rhel | arch | suse) seen "/etc/cron.d/backup.job, which cronie runs" "/usr/local/bin/backup" cron.d-dotted ;;
    alpine)
        seen "/etc/periodic/daily/sweep, which crond runs through run-parts" "/etc/periodic/daily/sweep" periodic
        seen "/etc/local.d/beacon.start, which OpenRC runs at boot" "/etc/local.d/beacon.start" local.d
        ;;
    esac
}

matrix_lines() {
    sed -n 's/^  collector \([^:]*\): \(ok\|degraded\|unavailable\|off\)\(.*\)$/MATRIX	\1	\2	\3/p' "$1" \
        | sed 's/	 — /	/'
}

inside() {
    [ -n "${TARGET:-}" ] || { echo "distributions: TARGET is not set" >&2; exit 2; }
    # shellcheck disable=SC1091
    . /etc/os-release
    # shellcheck disable=SC2086
    family="$(family_of "$ID" ${ID_LIKE:-})"
    say "${PRETTY_NAME:-$ID} — family $family, $(uname -m), kernel $(uname -r)"

    say "packages this run needs"
    prepare "$family"

    say "vigil"
    install_vigil "$family"
    vigild --version

    say "the host, made to look used"
    shape_the_host "$family"
    dump_like_the_timers

    say "vigild configure --dry-run"
    { vigild configure --dry-run >/dev/null; } 2>&1 || true

    say "vigild, for $((ROUND_SECONDS * 2)) seconds"
    SCHEDULE=2 sh /work/env/scripts/config-here.sh /tmp/vigil-config >/dev/null
    printf 'reporters:\n  - kind: ndjson\n    path: /tmp/findings.ndjson\n' > /tmp/vigil-config/reporters/local.yaml
    chmod 0600 /tmp/vigil-config/reporters/local.yaml
    vigild /tmp/vigil-config/vigil.yaml 2>/tmp/vigild.log &
    daemon=$!

    waited=0
    while [ ! -S /run/vigil/vigil.sock ] && [ "$waited" -lt 30 ]; do
        sleep 1
        waited=$((waited + 1))
    done
    sleep "$ROUND_SECONDS"

    useradd -m -s /bin/sh bob
    usermod -a -G "$(privileged_group)" bob
    echo 'bob ALL=(ALL) ALL' > /etc/sudoers.d/91-bob
    chmod 0440 /etc/sudoers.d/91-bob
    echo '*/1 * * * * root /tmp/.x/beacon' >> /etc/cron.d/vigil-check
    printf '*/2 * * * * /tmp/.y/other\n' | crontab -u bob - 2>/dev/null || true
    sleep "$ROUND_SECONDS"

    say "vigil capture"
    vigil capture --screen summary 2>&1 || true

    kill -TERM "$daemon" 2>/dev/null || true
    wait "$daemon" 2>/dev/null || true

    say "what vigild said"
    cat /tmp/vigild.log

    say "findings by kind"
    sed -n 's/^  \[[a-z]*\] \([a-z_.]*\) — .*/\1/p' /tmp/vigild.log | sort | uniq -c
    if [ -s /tmp/findings.ndjson ]; then
        echo "$(wc -l < /tmp/findings.ndjson) line(s) reached the ndjson reporter at /tmp/findings.ndjson"
    else
        echo "nothing reached the ndjson reporter at /tmp/findings.ndjson"
    fi

    say "what the readings hold of the host made to look used"
    expected "$family"

    matrix_lines /tmp/vigild.log
    cat /tmp/expected
}

case "${1:-run}" in
build) build ;;
run) shift 2>/dev/null || true; run "$@" ;;
inside) inside ;;
*)
    echo "usage: env/scripts/distributions.sh {build|run [image...]|inside}" >&2
    exit 2
    ;;
esac
