#!/usr/bin/env bash

set -uo pipefail

cd "$(dirname "$0")/../.."

KEY=vigil_exec
ACTION=always,exit
RULE=(-F arch=b64 -S execve -F "auid>=1000" -F "auid!=unset" -k "$KEY")
SPOOL=/var/lib/vigil/audit-spool
PLUGIN_CONFIG=/etc/audit/plugins.d/vigil.conf
PROBE=/usr/local/bin/vigil-audit-probe
CONFIG=/tmp/vigil-audit.yaml
FINDINGS=/tmp/vigil-audit-findings.ndjson
BUILT="${CARGO_TARGET_DIR:-$PWD/target}/debug"

failures=0
ok()  { printf '  ok    %s\n' "$*"; }
bad() { printf '  FAIL  %s\n' "$*"; failures=$((failures + 1)); }
say() { printf '\n\033[1m== %s\033[0m\n' "$*"; }

cat <<'NOTE'

This check changes the audit state of the KERNEL, and the kernel is not the container's:
audit has no namespace. Under Docker Desktop that is the virtual machine's kernel; on a Linux
workstation it is this machine's own. What it puts in, it takes out again — its own rule by
the same specification it added, its own auditd, and the enable flag as it found it — and it
does that on the way out however it leaves.

NOTE

enabled_now() { auditctl -s 2>/dev/null | awk '/^enabled/ {print $2}'; }
registered_pid() { auditctl -s 2>/dev/null | awk '/^pid/ {print $2}'; }
alive() { [ -n "${1:-}" ] && [ "$1" != "0" ] && [ -d "/proc/$1" ]; }

say "the machine, and whether this is a machine we may touch"
uname -sr
command -v auditctl >/dev/null || { echo "no auditctl here: this recipe needs the audit package" >&2; exit 2; }
auditctl -v 2>&1 | head -1

if ! auditctl -s >/dev/null 2>&1; then
    echo "the audit netlink socket is closed to this container." >&2
    echo "It opens with --pid host; the capabilities alone are not enough." >&2
    exit 2
fi

theirs="$(registered_pid)"
if alive "$theirs"; then
    echo >&2
    echo "an auditd is already running and registered with this kernel, at pid $theirs, and" >&2
    echo "it is not ours. There is one audit daemon per kernel: taking it over would stop" >&2
    echo "whatever collects audit on this machine, and this recipe will not do that to" >&2
    echo "somebody's working host. Stop it deliberately, or run this somewhere else." >&2
    echo >&2
    echo "  what it is:  $(tr '\0' ' ' < "/proc/$theirs/cmdline" 2>/dev/null)" >&2
    exit 2
fi
case "${theirs:-0}" in
0) ok "no auditd is registered with this kernel, so there is nothing here to displace" ;;
*) ok "pid $theirs is registered and is not running: a registration an earlier auditd left behind, not a daemon to displace" ;;
esac

WAS_ENABLED="$(enabled_now)"
WAS_ENABLED="${WAS_ENABLED:-0}"
BEFORE="$(auditctl -l 2>/dev/null)"
ok "audit is enabled=$WAS_ENABLED and holds $(printf '%s' "$BEFORE" | grep -c 'always,exit\|never\|-w ') rule(s); both go back as they are"

daemon=0
auditor=0
put_it_back() {
    [ "$daemon" -ne 0 ] && kill "$daemon" 2>/dev/null
    auditctl -d "$ACTION" "${RULE[@]}" >/dev/null 2>&1
    if [ "$auditor" -ne 0 ]; then
        kill -TERM "$auditor" 2>/dev/null
        for _ in 1 2 3 4 5 6 7 8 9 10; do
            alive "$auditor" || break
            sleep 1
        done
    fi
    auditctl -e "$WAS_ENABLED" >/dev/null 2>&1
    rm -f "$PLUGIN_CONFIG" "$SPOOL" "$SPOOL.cursor" "$SPOOL.writing" "$PROBE"
}
trap put_it_back EXIT INT TERM

say "build"
cargo build --workspace --quiet || exit 1

say "the plugin auditd will start, named by its absolute path"
install -d -m 0750 /etc/audit/plugins.d
cat > "$PLUGIN_CONFIG" <<PLUGIN
active = yes
direction = out
path = $BUILT/vigil-audit-plugin
type = always
format = string
PLUGIN
chmod 0640 "$PLUGIN_CONFIG"
sed 's/^/  /' "$PLUGIN_CONFIG"
[ -x "$BUILT/vigil-audit-plugin" ] && ok "the plugin is built and executable" \
    || { bad "no plugin at $BUILT/vigil-audit-plugin"; exit 1; }

rm -f "$SPOOL" "$SPOOL.cursor" "$SPOOL.writing" "$FINDINGS"
install -d -m 0700 /var/lib/vigil

say "a real auditd, started here, with our rule in the kernel"
auditd
for _ in 1 2 3 4 5 6 7 8 9 10; do
    started="$(registered_pid)"
    alive "$started" && break
    sleep 1
done
if ! alive "${started:-0}"; then
    bad "auditd did not come up and register with the kernel"
    exit 1
fi
if [ "$started" = "$theirs" ]; then
    bad "the registered pid is the one that was there before this ran: that daemon is not ours to stop"
    exit 1
fi
auditor="$started"
ok "the daemon this recipe started is at pid $auditor, and the kernel confirms it: that pid, and no other, is the one it may stop"

auditctl -e 1 >/dev/null
auditctl -a "$ACTION" "${RULE[@]}" >/dev/null || bad "the kernel would not take the rule this product ships"
auditctl -l | sed 's/^/  /'
auditctl -l | grep -q "$KEY" && ok "the rule is loaded and tagged $KEY" || bad "the rule is not in the kernel"

mine="$(registered_pid)"
[ "$mine" = "$auditor" ] \
    && ok "the kernel still hands audit to that pid and to no other" \
    || bad "the registered pid is $mine and the daemon we started is $auditor"

say "a person runs a program: loginuid 1000, so the shipped rule is about them"
id tester >/dev/null 2>&1 || adduser -D -u 1000 tester >/dev/null 2>&1
install -m 0755 "$BUILT/vigil-audit-plugin" "$PROBE"

say "the agent, reading what the plugin leaves in the spool"
cat > "$CONFIG" <<YAML
state_dir: /tmp/vigil-audit-state
socket_path: /tmp/vigil-audit.sock
retention_days: 1
interval_seconds: 3
collectors: [launches]
schedule: {launches: 3}
suppressions: []
record_launch_arguments: false
reporters:
  - kind: ndjson
    path: $FINDINGS
YAML
rm -rf /tmp/vigil-audit-state
"$BUILT/vigild" "$CONFIG" > /tmp/vigil-audit-daemon.log 2>&1 &
daemon=$!
sleep 6
kill -0 "$daemon" 2>/dev/null || { bad "the daemon stopped on its own"; cat /tmp/vigil-audit-daemon.log; exit 1; }

sh -c "echo 1000 > /proc/self/loginuid && exec $PROBE --version" >/dev/null
ran=$?
[ "$ran" -eq 0 ] && ok "$PROBE ran with loginuid 1000" || bad "the probe would not run ($ran)"

say "the kernel wrote it, auditd handed it over, the plugin spooled it"
for _ in $(seq 1 15); do
    [ -s "$SPOOL" ] && grep -q "$PROBE" "$SPOOL" && break
    sleep 1
done
if [ -s "$SPOOL" ] && grep -q "$PROBE" "$SPOOL"; then
    ok "the spool carries the execve: $(grep -o "exe=\"[^\"]*\"" "$SPOOL" | tail -1)"
    ok "$(wc -c < "$SPOOL") bytes in $SPOOL, written by the plugin auditd started"
else
    bad "nothing about $PROBE reached $SPOOL"
    ls -l "$SPOOL" 2>/dev/null
    tail -5 /var/log/audit/audit.log 2>/dev/null | sed 's/^/  audit.log: /'
fi
if grep -q "PROCTITLE" "$SPOOL" 2>/dev/null; then
    bad "the spool holds record types this product never reads"
else
    ok "and nothing but the record types the collector reads reached the disk"
fi

say "the collector read the spool, the rule judged it, the reporter carried it"
for _ in $(seq 1 15); do
    grep -q "exec\." "$FINDINGS" 2>/dev/null && break
    sleep 2
done
if grep -q "exec.first_seen_for_user" "$FINDINGS" 2>/dev/null; then
    ok "$(grep -o '"kind":"exec[^"]*"' "$FINDINGS" | tail -1) — $(grep -o '"title":"[^"]*"' "$FINDINGS" | tail -1)"
else
    bad "the launch never became a finding a person ran a program for the first time"
    grep -o '"kind":"[^"]*"' "$FINDINGS" 2>/dev/null | sort -u | sed 's/^/  /'
    tail -20 /tmp/vigil-audit-daemon.log | sed 's/^/  /'
fi
grep -q "$PROBE" "$FINDINGS" 2>/dev/null \
    && ok "and the finding names the program that was run" \
    || bad "the finding does not name $PROBE"
source_row="$(printf '%s\n' '{"query":"snapshot","collector":"launches"}' | nc -U -q 2 /tmp/vigil-audit.sock 2>/dev/null \
    | grep -o '"launches|source":{[^}]*}')"
case "$source_row" in
*"audit plugin"*)
    ok "and the reading says where it came from: $source_row" ;;
*)
    bad "the collector did not read the plugin's spool: ${source_row:-nothing came back}" ;;
esac

standing="$(printf '%s\n' '{"query":"status"}' | nc -U -q 2 /tmp/vigil-audit.sock 2>/dev/null \
    | grep -o '"name":"launches","state":"[a-z]*"')"
case "$standing" in
*'"state":"unavailable"'*|"")
    bad "the collector says it has no source at all after a real launch went through it: ${standing:-nothing came back}" ;;
*)
    ok "and the collector has a source it is reading: $standing" ;;
esac

say "putting the kernel back the way it was found"
put_it_back
trap - EXIT INT TERM
sleep 1
after_enabled="$(enabled_now)"
after_rules="$(auditctl -l 2>/dev/null)"
after_pid="$(registered_pid)"

[ "$after_enabled" = "$WAS_ENABLED" ] \
    && ok "audit is enabled=$after_enabled again, as it was" \
    || bad "audit was enabled=$WAS_ENABLED and is now enabled=$after_enabled"
if [ "$after_rules" = "$BEFORE" ]; then
    ok "the rule list is what it was before this ran"
else
    bad "the rule list changed under this run"
    printf '  before: %s\n  after:  %s\n' "$BEFORE" "$after_rules"
fi
if alive "$auditor"; then
    bad "the daemon this recipe started is still running at pid $auditor: it left one behind"
else
    ok "the daemon this recipe started, pid $auditor, is gone"
fi
if [ "$after_pid" = "$auditor" ] && alive "$after_pid"; then
    bad "the kernel still hands audit to the daemon this recipe started"
else
    ok "the kernel registers no running daemon of ours any more (pid reads ${after_pid:-0})"
fi

say "result"
if [ "$failures" -eq 0 ]; then
    echo "A live auditd handed a real execve to the plugin, the plugin spooled it, the"
    echo "collector read the spool and the agent reported it: kernel → auditd → plugin →"
    echo "spool → collector → rule → reporter, end to end, and the kernel is as it was found."
    exit 0
fi
echo "$failures check(s) failed" >&2
exit 1
