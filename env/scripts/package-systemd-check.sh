#!/usr/bin/env bash
set -uo pipefail

fail=0
ok()  { printf '  ok    %s\n' "$*"; }
bad() { printf '  FAIL  %s\n' "$*"; fail=$((fail + 1)); }
say() { printf '\n\033[1m== %s\033[0m\n' "$*"; }

DEB="${DEB:-$(ls /work/dist/vigil_*_amd64.deb 2>/dev/null | grep -v -- '-upgrade' | head -1)}"
DROPIN=/etc/systemd/system/vigild.service.d

ask()   { printf '%s\n' "$1" | nc -U -q 2 /run/vigil/vigil.sock 2>/dev/null; }
ports() { ask '{"query":"snapshot","collector":"ports"}'; }
users() { ask '{"query":"snapshot","collector":"users"}'; }
listener() { ports | sed 's/"tcp|/\n"tcp|/g' | grep '^"tcp|0.0.0.0:4444"'; }

say "the machine"
systemctl --version | head -1
[ -d /run/systemd/system ] && ok "systemd is pid 1" || { bad "no systemd here"; exit 1; }
[ -f "$DEB" ] || { echo "no package to install (looked in /work/dist)" >&2; exit 2; }

say "an unprivileged user, a key in a 0700 home, and a socket that user holds"
id tester >/dev/null 2>&1 || useradd -m -s /bin/sh tester
mkdir -p /home/tester/.ssh
echo 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIMK7uPzOaGNIWRRT4Pk6mFPFbLNjmHkCoyCiPWvKvqYt tester@probe' \
    > /home/tester/.ssh/authorized_keys
chown -R tester:tester /home/tester
chmod 0700 /home/tester /home/tester/.ssh
chmod 0600 /home/tester/.ssh/authorized_keys
stat -c '  %n is %a %U:%G' /home/tester /etc/shadow

say "install, and a configuration that reads often enough to watch"
dpkg -i "$DEB" >/dev/null || exit 1
cat > /etc/vigil/vigil.yaml <<'YAML'
state_dir: /var/lib/vigil
socket_path: /run/vigil/vigil.sock
interval_seconds: 3
retention_days: 90
suppressions: []
reporters:
  - kind: ndjson
    path: /var/log/vigil/findings.ndjson
YAML
chmod 0600 /etc/vigil/vigil.yaml

say "systemctl enable --now vigild"
systemctl enable --now vigild >/dev/null 2>&1
sleep 6
[ "$(systemctl is-active vigild)" = active ] \
    && ok "the service is active" \
    || { bad "the service is $(systemctl is-active vigild)"; systemctl status vigild --no-pager -l | tail -25; }
systemctl show vigild -p User -p CapabilityBoundingSet -p ProtectHome -p ProtectSystem | sed 's/^/  /'

say "the directories the unit declares"
for path in /run/vigil /var/lib/vigil /var/log/vigil /run/vigil/vigil.sock; do
    mode="$(stat -c '%a %U:%G' "$path" 2>/dev/null)"
    case "$path:$mode" in
    */vigil.sock:"600 root:root") ok "$path  $mode" ;;
    */vigil.sock:*)               bad "$path is $mode, expected 600 root:root" ;;
    *:"700 root:root")            ok "$path  $mode" ;;
    *)                            bad "$path is ${mode:-missing}, expected 700 root:root" ;;
    esac
done

say "the chain, under the hardened unit: a port opened next door comes out as a finding"
setsid su tester -s /bin/sh -c 'exec nc -l 4444' >/dev/null 2>&1 &
sleep 2
ps -o pid,user,args --no-headers -p "$(pgrep -u tester -x nc | head -1)" | sed 's/^/  listener: /'
for _ in 1 2 3 4 5 6 7 8 9 10; do
    grep -q 'port.listen|tcp|0.0.0.0:4444' /var/log/vigil/findings.ndjson 2>/dev/null && break
    sleep 2
done
grep -q 'port.listen|tcp|0.0.0.0:4444' /var/log/vigil/findings.ndjson 2>/dev/null \
    && ok "$(grep -o '"finding_key":"port[^"]*"' /var/log/vigil/findings.ndjson | tail -1)" \
    || bad "the port on :4444 never reached the ndjson reporter"

say "what each capability buys"
printf '  %-30s %-14s %-24s %s\n' 'CapabilityBoundingSet' 'CapEff' 'owner of :4444' "tester's ssh keys"
PROBE_RESULT=""
probe() {
    local label="$1" set="$2"
    mkdir -p "$DROPIN"
    { echo '[Service]'; echo 'CapabilityBoundingSet='; echo "CapabilityBoundingSet=$set"; } \
        > "$DROPIN/capabilities.conf"
    systemctl daemon-reload
    systemctl restart vigild
    sleep 7
    local pid effective resolved keys
    pid="$(systemctl show -p MainPID --value vigild)"
    effective="$(awk '/^CapEff/{print $2}' "/proc/$pid/status" 2>/dev/null)"
    resolved="$(listener | grep -o '"owner_resolved":[a-z]*' | head -1 | cut -d: -f2)"
    keys="$(users | grep -c 'sshkey|tester|SHA256')"
    printf '  %-30s %-14s %-24s %s\n' "${label}" "$effective" "${resolved:-not-seen}" "$keys"
    PROBE_RESULT="$resolved $keys"
}

probe 'nothing' '';                                              nothing="$PROBE_RESULT"
probe 'CAP_DAC_READ_SEARCH' 'CAP_DAC_READ_SEARCH';               dac="$PROBE_RESULT"
probe 'CAP_SYS_PTRACE' 'CAP_SYS_PTRACE';                         ptrace="$PROBE_RESULT"
probe 'both (as shipped)' 'CAP_DAC_READ_SEARCH CAP_SYS_PTRACE';  both="$PROBE_RESULT"

[ "$both" = "true 1" ] \
    && ok "with both, the agent resolves the socket's owner and reads the key" \
    || bad "with both, the agent still cannot see everything: [$both]"
[ "$nothing" = "false 0" ] \
    && ok "with neither, it can see neither — the capabilities are doing the work" \
    || bad "with no capabilities the agent saw [$nothing]: something else is granting this"
[ "$dac" != "true 1" ] \
    && ok "CAP_DAC_READ_SEARCH alone is not enough ([$dac])" \
    || bad "CAP_DAC_READ_SEARCH alone is enough — CAP_SYS_PTRACE should come out of the unit"
[ "$ptrace" != "true 1" ] \
    && ok "CAP_SYS_PTRACE alone is not enough ([$ptrace])" \
    || bad "CAP_SYS_PTRACE alone is enough — CAP_DAC_READ_SEARCH should come out of the unit"
rm -f "$DROPIN/capabilities.conf"

say "ProtectHome=yes, the value the unit deliberately does not use"
mkdir -p "$DROPIN"
printf '[Service]\nProtectHome=yes\n' > "$DROPIN/home.conf"
systemctl daemon-reload; systemctl restart vigild; sleep 7
blinded="$(users | grep -c 'sshkey|tester|SHA256')"
rm -f "$DROPIN/home.conf"
systemctl daemon-reload; systemctl restart vigild; sleep 7
seeing="$(users | grep -c 'sshkey|tester|SHA256')"
if [ "$blinded" = 0 ] && [ "$seeing" = 1 ]; then
    ok "ProtectHome=yes hides every authorized_keys on the machine ($blinded seen); read-only shows them ($seeing)"
else
    bad "expected 0 keys under ProtectHome=yes and 1 under read-only, got $blinded and $seeing"
fi

say "systemctl stop"
start=$(date +%s)
systemctl stop vigild
took=$(( $(date +%s) - start ))
[ "$took" -lt 5 ] && ok "stopped in ${took}s on SIGTERM" || bad "took ${took}s — SIGTERM was not enough"
journalctl -u vigild --no-pager 2>/dev/null | grep -qiE 'killed with signal|timed out' \
    && bad "the journal says it had to be killed" || ok "nothing in the journal about a kill"

say "result"
if [ "$fail" -eq 0 ]; then
    echo "the unit runs the agent under a real systemd, with two capabilities and no more,"
    echo "and the agent can still see everything it is supposed to see."
    exit 0
fi
echo "$fail check(s) failed" >&2
exit 1
