#!/usr/bin/env bash
set -uo pipefail

cd "$(dirname "$0")/../.."
FORMAT="${1:-deb}"
OUT="${OUT:-$PWD/dist}"
VERSION="${VERSION:-$(awk '/^\[workspace\.package\]/{p=1} p && /^version *= *"/{gsub(/[^0-9A-Za-z.+~-]/,"",$3); print $3; exit}' Cargo.toml)}"
NEXT="${NEXT:-0.0.0}"

failures=0
say() { printf '\n\033[1m== %s\033[0m\n' "$*"; }
ok() { printf '  ok    %s\n' "$*"; }
bad() { printf '  FAIL  %s\n' "$*"; failures=$((failures + 1)); }

expect_mode() {
    local path="$1" mode="$2" owner="${3:-root:root}"
    if [ ! -e "$path" ]; then bad "$path is missing"; return; fi
    local actual; actual="$(stat -c '%a %U:%G' "$path")"
    if [ "$actual" = "$mode $owner" ]; then ok "$path  $actual"
    else bad "$path is $actual, expected $mode $owner"; fi
}

expect_absent() {
    if [ -e "$1" ]; then bad "$1 is still here and should not be"; else ok "$1 is gone"; fi
}

expect_present() {
    if [ -e "$1" ]; then ok "$1 survived"; else bad "$1 was removed and should not have been"; fi
}

case "$FORMAT" in
deb) PACKAGE="$OUT/vigil_${VERSION}_$(dpkg --print-architecture).deb" ;;
rpm) PACKAGE="$OUT/vigil-${VERSION}-1.$(uname -m).rpm" ;;
*) echo "usage: env/scripts/package-check.sh {deb|rpm}" >&2; exit 2 ;;
esac

say "the machine this is being installed on"
cat /etc/os-release | grep -E '^(NAME|VERSION)=' || true
echo "arch: $(uname -m)   kernel: $(uname -r)"
echo "package: $PACKAGE"
[ -f "$PACKAGE" ] || { echo "no such package — build it first" >&2; exit 2; }

for leftover in /usr/sbin/vigild /usr/bin/vigil /usr/sbin/vigil-audit-plugin /usr/sbin/vigil-container-dump /etc/vigil /var/lib/vigil; do
    [ -e "$leftover" ] && { echo "this machine is not clean: $leftover exists" >&2; exit 2; }
done

say "install"
case "$FORMAT" in
deb) dpkg -i "$PACKAGE" || exit 1 ;;
rpm) rpm -i "$PACKAGE" || exit 1 ;;
esac

say "what it put where"
expect_mode /usr/sbin/vigild 755
expect_mode /usr/bin/vigil 755
expect_mode /usr/sbin/vigil-audit-plugin 755
expect_mode /usr/sbin/vigil-container-dump 755
expect_mode /etc/vigil 700
expect_mode /etc/vigil/vigil.yaml 600
expect_mode /etc/logrotate.d/vigil 644
expect_mode /usr/lib/systemd/system/vigild.service 644
expect_mode /usr/lib/systemd/system/vigil-containers.service 644
expect_mode /usr/lib/systemd/system/vigil-containers.timer 644
expect_mode /usr/lib/tmpfiles.d/vigil.conf 644
expect_mode /var/lib/vigil 700
expect_mode /var/lib/vigil/containers 700
expect_mode /var/log/vigil 700
expect_mode /run/vigil 700
expect_mode /etc/audit 750
expect_mode /etc/audit/rules.d 750
expect_mode /etc/audit/plugins.d 750
expect_mode /etc/audit/rules.d/vigil-exec.rules 640
expect_mode /etc/audit/plugins.d/vigil.conf 640

say "the shipped configuration is the example, unedited"
if [ "$(md5sum < config/vigil.example.yaml)" = "$(md5sum < /etc/vigil/vigil.yaml)" ]; then
    ok "/etc/vigil/vigil.yaml is config/vigil.example.yaml"
else
    bad "the installed configuration is not the shipped example"
fi

say "the container dump, on a machine with no engine installed"
if /usr/sbin/vigil-container-dump --version | grep -q "vigil-container-dump $VERSION"; then
    ok "$(/usr/sbin/vigil-container-dump --version)"
else
    bad "vigil-container-dump --version says '$(/usr/sbin/vigil-container-dump --version 2>&1)'"
fi
if /usr/sbin/vigil-container-dump >/dev/null 2>&1 \
    && grep -q '"state": "absent"' /var/lib/vigil/containers/docker.json; then
    ok "an engine that is not here is written down as absent rather than left out"
else
    bad "the dump wrote no document for an engine this machine does not have"
fi
expect_mode /var/lib/vigil/containers/docker.json 600

say "the audit plugin, fed a recorded event the way auditd would feed a live one"
if /usr/sbin/vigil-audit-plugin --version | grep -q "vigil-audit-plugin $VERSION"; then
    ok "$(/usr/sbin/vigil-audit-plugin --version)"
else
    bad "vigil-audit-plugin --version says '$(/usr/sbin/vigil-audit-plugin --version 2>&1)'"
fi

recorded=/tmp/recorded-audit.log
cat > "$recorded" <<'RECORD'
type=SYSCALL msg=audit(1757419203.412:3421): arch=c000003e syscall=59 success=yes exit=0 items=2 ppid=2143 pid=2170 auid=0 uid=0 tty=pts0 ses=3 comm="id" exe="/usr/bin/id" key="vigil_exec"
type=EXECVE msg=audit(1757419203.412:3421): argc=1 a0="id"
type=PROCTITLE msg=audit(1757419203.412:3421): proctitle=2F7573722F62696E2F6964
type=USER_LOGIN msg=audit(1757419203.400:3400): pid=1 uid=0 auid=0 res=success
RECORD

if /usr/sbin/vigil-audit-plugin < "$recorded" 2>&1 | grep -q "2 record(s) kept"; then
    ok "the plugin kept the two records this product reads"
else
    bad "the plugin did not keep what it should: $(/usr/sbin/vigil-audit-plugin < "$recorded" 2>&1)"
fi
expect_mode /var/lib/vigil/audit-spool 600
if grep -q "vigil_exec" /var/lib/vigil/audit-spool; then
    ok "the launch is in the spool"
else
    bad "the spool holds no launch"
fi

if grep -q "PROCTITLE\|USER_LOGIN" /var/lib/vigil/audit-spool; then
    bad "the spool holds records this product never reads"
else
    ok "nothing but the records the collector reads reached the disk"
fi

say "it runs"
if /usr/sbin/vigild --version | grep -q "vigild $VERSION"; then
    ok "$(/usr/sbin/vigild --version)"
else
    bad "vigild --version says '$(/usr/sbin/vigild --version 2>&1)'"
fi
if /usr/bin/vigil --version >/dev/null 2>&1 || /usr/bin/vigil --help >/dev/null 2>&1; then
    ok "the console binary executes"
else
    bad "the console binary does not execute"
fi

say "it reads the installed configuration and opens its socket"

( umask 0077; exec /usr/sbin/vigild /etc/vigil/vigil.yaml ) >/tmp/vigild.out 2>&1 &
daemon=$!
for _ in 1 2 3 4 5 6 7 8 9 10; do
    [ -S /run/vigil/vigil.sock ] && break
    sleep 1
done

if [ -S /run/vigil/vigil.sock ]; then
    expect_mode /run/vigil/vigil.sock 600
else
    bad "no socket at /run/vigil/vigil.sock after ten seconds"
fi
grep -q 'host ' /tmp/vigild.out && ok "identity: $(head -1 /tmp/vigild.out)" \
    || bad "the daemon said nothing about the host it is watching"
grep -q 'collector ports: ok' /tmp/vigild.out && ok "the socket collector is healthy" \
    || bad "the socket collector is not ok: $(grep -i collector /tmp/vigild.out | tr '\n' ' ')"
grep -q 'collector launches: ok' /tmp/vigild.out \
    && ok "the launches collector is reading the plugin's spool" \
    || bad "the launches collector is not ok: $(grep -i launches /tmp/vigild.out | tr '\n' ' ')"

say "it stops on SIGTERM without being killed"
kill -TERM "$daemon" 2>/dev/null
for _ in 1 2 3 4 5; do kill -0 "$daemon" 2>/dev/null || break; sleep 1; done
if kill -0 "$daemon" 2>/dev/null; then
    bad "still running five seconds after SIGTERM"
    kill -KILL "$daemon" 2>/dev/null
else
    ok "gone on SIGTERM"
fi
wait "$daemon" 2>/dev/null

say "the state directory has something in it, and it is 0600"
expect_mode /var/lib/vigil 700
expect_mode /var/lib/vigil/containers 700
expect_mode /var/lib/vigil/install_id 600
expect_mode /var/lib/vigil/audit-spool.cursor 600
install_id="$(cat /var/lib/vigil/install_id 2>/dev/null || true)"
[ -n "$install_id" ] && ok "install_id $install_id" || bad "no install_id was minted"

say "upgrade to $NEXT with a locally edited configuration"
printf '\n# edited by the operator on this machine\nretention_days: 7\n' >> /etc/vigil/vigil.yaml
edited="$(md5sum /etc/vigil/vigil.yaml | cut -d' ' -f1)"

case "$FORMAT" in
deb)
    next_package="$OUT/vigil_${NEXT}_$(dpkg --print-architecture).deb"
    [ -f "$next_package" ] && dpkg -i "$next_package" || echo "(no $next_package — upgrade not exercised)"
    ;;
rpm)
    next_package="$OUT/vigil-${NEXT}-1.$(uname -m).rpm"
    [ -f "$next_package" ] && rpm -U "$next_package" || echo "(no $next_package — upgrade not exercised)"
    ;;
esac

if [ -f "$next_package" ]; then
    if [ "$(md5sum /etc/vigil/vigil.yaml | cut -d' ' -f1)" = "$edited" ]; then
        ok "the edited /etc/vigil/vigil.yaml survived the upgrade"
    else
        bad "the upgrade overwrote /etc/vigil/vigil.yaml"
    fi
    expect_mode /etc/vigil/vigil.yaml 600
    case "$FORMAT" in
    deb) installed="$(dpkg-query -W -f '${Version}' vigil)" ;;
    rpm) installed="$(rpm -q --queryformat '%{VERSION}' vigil)" ;;
    esac
    [ "$installed" = "$NEXT" ] \
        && ok "the installed package is now $installed" \
        || bad "the installed package is $installed, expected $NEXT"
    expect_present /var/lib/vigil/install_id
    [ "$(cat /var/lib/vigil/install_id)" = "$install_id" ] \
        && ok "install_id unchanged — an upgrade is not a new installation" \
        || bad "install_id changed across the upgrade"
fi

printf '{"kind":"port.listen.new"}\n' > /var/log/vigil/findings.ndjson
chmod 0600 /var/log/vigil/findings.ndjson

say "remove"
case "$FORMAT" in
deb)
    dpkg -r vigil
    expect_absent /usr/sbin/vigild
    expect_absent /usr/bin/vigil
    expect_absent /usr/sbin/vigil-audit-plugin
    expect_absent /usr/sbin/vigil-container-dump
    expect_present /etc/vigil/vigil.yaml
    expect_present /etc/audit/rules.d/vigil-exec.rules
    expect_present /var/lib/vigil/install_id
    ;;
rpm)
    rpm -e vigil
    expect_absent /usr/sbin/vigild
    expect_absent /usr/bin/vigil
    expect_present /var/lib/vigil/install_id
    expect_present /var/log/vigil/findings.ndjson
    ;;
esac

if [ "$FORMAT" = deb ]; then
    say "purge"
    dpkg -P vigil
    expect_absent /etc/vigil
    expect_absent /etc/audit/rules.d/vigil-exec.rules
    expect_absent /etc/audit/plugins.d/vigil.conf
    expect_present /var/lib/vigil/install_id
    expect_present /var/log/vigil/findings.ndjson
fi

say "result"
if [ "$failures" -eq 0 ]; then
    echo "clean machine, package installed, agent ran, upgrade kept the configuration,"
    echo "removal kept the findings. Nothing outside this container was contacted."
    echo
    echo "NOT checked here (no systemd as pid 1 in a container): systemctl enable/start, and"
    echo "every sandbox directive in the unit. See 'just package-unit'."
    exit 0
fi
echo "$failures check(s) failed" >&2
exit 1
