#!/usr/bin/env bash
set -uo pipefail

fail=0
ok()  { printf '  ok    %s\n' "$*"; }
bad() { printf '  FAIL  %s\n' "$*"; fail=$((fail + 1)); }
say() { printf '\n\033[1m== %s\033[0m\n' "$*"; }

DEB="${DEB:-$(ls -t /work/dist/vigil_*_amd64.deb 2>/dev/null | grep -v -- '-upgrade' | head -1)}"
DROPIN=/etc/systemd/system/vigild.service.d

ask()   { printf '%s\n' "$1" | nc -U -q 2 /run/vigil/vigil.sock 2>/dev/null; }
network() { ask '{"query":"snapshot","collector":"network"}'; }
users() { ask '{"query":"snapshot","collector":"users"}'; }
listener() { network | sed 's/"tcp|/\n"tcp|/g' | grep '^"tcp|0.0.0.0:4444"'; }

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

say "the firewall reading: a unit runs nft, the agent reads the file it leaves"
systemctl start vigil-firewall.service
sleep 2
ruleset=/var/lib/vigil/firewall/ruleset.json
mode="$(stat -c '%a %U:%G' "$ruleset" 2>/dev/null)"
[ "$mode" = "600 root:root" ] \
    && ok "$ruleset  $mode" \
    || bad "$ruleset is ${mode:-missing}, expected 600 root:root"
if [ -s "$ruleset" ] && head -c 40 "$ruleset" | grep -q '"nftables"'; then
    ok "$(head -c 110 "$ruleset")"
else
    bad "the unit left nothing readable in $ruleset: $(head -c 200 "$ruleset" 2>/dev/null)"
fi
nft add table inet vigil-probe 2>/dev/null
nft add chain inet vigil-probe input "{ type filter hook input priority 0 ; policy drop ; }" 2>/dev/null
systemctl start vigil-firewall.service
sleep 2
if grep -q 'vigil-probe' "$ruleset"; then
    ok "a table added between two runs of the timer is in the next reading"
else
    bad "the table added on this host never reached $ruleset"
fi
nft delete table inet vigil-probe 2>/dev/null
if [ "$(systemctl show vigild -p Wants --value | tr ' ' '\n' | grep -c vigil-firewall.timer)" = 1 ]; then
    ok "vigild pulls vigil-firewall.timer in"
else
    bad "vigild does not want vigil-firewall.timer, so enabling the agent leaves the firewall unread"
fi
systemctl mask vigil-firewall.timer >/dev/null 2>&1
if [ "$(systemctl is-enabled vigil-firewall.timer 2>/dev/null)" = masked ]; then
    ok "systemctl mask vigil-firewall.timer switches the privileged half off"
else
    bad "the timer cannot be masked, so the only way to stop the reading is to remove the package"
fi
systemctl unmask vigil-firewall.timer >/dev/null 2>&1

say "vigild collector firewall enable|disable, the command an operator types"
systemctl disable --now vigil-firewall.timer >/dev/null 2>&1
SWITCH=/tmp/vigil-switch
rm -rf "$SWITCH"
mkdir -p "$SWITCH/collectors"
sed 's#/etc/vigil/#'"$SWITCH"'/#g' /work/config/vigil.example.yaml > "$SWITCH/vigil.yaml"
for shipped in /work/config/collectors/*.yaml; do
    sed 's#/etc/vigil/#'"$SWITCH"'/#g' "$shipped" > "$SWITCH/collectors/$(basename "$shipped")"
done
cp /work/config/watch_fs.yaml "$SWITCH/watch_fs.yaml"
BLOCK="$SWITCH/collectors/firewall.yaml"
sed -i 's/^firewall:$/firewall:\n  enabled: false/' "$BLOCK"
printf '\n# a line of the administrator\x27s own, which this command must not move\n' >> "$BLOCK"
kept="$(md5sum < "$BLOCK")"

/usr/sbin/vigild collector firewall enable --config "$SWITCH/vigil.yaml" 2>&1 | sed 's/^/  /'
grep -q '^  enabled: true$' "$BLOCK" \
    && ok "the block says enabled: true" || bad "the block does not say enabled: true"
grep -q '^  schedule: 60$' "$BLOCK" \
    && ok "and its period is where it was" || bad "the period of the block moved"
grep -q "administrator" "$BLOCK" \
    && ok "and the administrator's own line is untouched" \
    || bad "a line this command did not write is gone"
[ "$(systemctl is-enabled vigil-firewall.timer)" = enabled ] \
    && ok "the timer is enabled" || bad "the timer is $(systemctl is-enabled vigil-firewall.timer)"
[ -f "$BLOCK.previous" ] \
    && ok "and what was there is kept as $BLOCK.previous" \
    || bad "the previous file was not kept"

after="$(md5sum < "$BLOCK")"
/usr/sbin/vigild collector firewall enable --config "$SWITCH/vigil.yaml" 2>&1 | sed 's/^/  /'
if [ "$(md5sum < "$BLOCK")" = "$after" ]; then
    ok "running it again changes not one byte"
else
    bad "a second run edited the file again"
fi

systemctl mask vigil-firewall.timer >/dev/null 2>&1
if /usr/sbin/vigild collector firewall enable --config "$SWITCH/vigil.yaml" >/tmp/masked.out 2>&1; then
    bad "it walked past a masked unit: $(cat /tmp/masked.out)"
else
    grep -q "masked" /tmp/masked.out \
        && ok "a masked timer stops it, and it says so" \
        || bad "$(cat /tmp/masked.out)"
fi
[ "$(systemctl is-enabled vigil-firewall.timer)" = masked ] \
    && ok "and it did not unmask anything behind the administrator" \
    || bad "the mask is gone"
systemctl unmask vigil-firewall.timer >/dev/null 2>&1

/usr/sbin/vigild collector firewall disable --config "$SWITCH/vigil.yaml" 2>&1 | sed 's/^/  /'
grep -q '^  enabled: false$' "$BLOCK" \
    && ok "disable wrote enabled: false back into the block" \
    || bad "disable left the block switched on"
[ "$(md5sum < "$BLOCK")" = "$kept" ] \
    && ok "and the file is byte for byte the one it started as" \
    || { bad "disable left the file different from how enable found it"; diff "$BLOCK" "$BLOCK.previous"; }
[ "$(systemctl is-enabled vigil-firewall.timer)" = disabled ] \
    && ok "the timer is disabled again" \
    || bad "the timer is $(systemctl is-enabled vigil-firewall.timer)"

say "the legacy iptables backend, on the one distribution that still ships it"
nft flush ruleset
iptables-legacy -A INPUT -p tcp --dport 4444 -j DROP 2>/dev/null
registered="$(tr '\n' ' ' < /proc/net/ip_tables_names 2>/dev/null)"
if [ -n "$registered" ]; then
    ok "/proc/net/ip_tables_names names the tables the old backend registered: $registered"
else
    bad "iptables-legacy took a rule and /proc/net/ip_tables_names is still empty: the \
discriminator this collector rests on does not work here"
fi
systemctl start vigil-firewall.service
sleep 2
mkdir -p /tmp/vigil-legacy
cat > /tmp/vigil-legacy/vigil.yaml <<'YAML'
state_dir: /tmp/vigil-legacy
socket_path: /tmp/vigil-legacy/vigil.sock
retention_days: 1
interval_seconds: 3
collectors: [firewall]
schedule: {firewall: 3}
suppressions: []
reporters: []
YAML
/usr/sbin/vigild /tmp/vigil-legacy/vigil.yaml > /tmp/vigil-legacy/log 2>&1 &
legacy=$!
sleep 7
said="$(printf '%s\n' '{"query":"status"}' | nc -U -q 2 /tmp/vigil-legacy/vigil.sock 2>/dev/null)"
reading="$(printf '%s\n' '{"query":"snapshot","collector":"firewall"}' | nc -U -q 2 /tmp/vigil-legacy/vigil.sock 2>/dev/null)"
kill "$legacy" 2>/dev/null
wait "$legacy" 2>/dev/null

case "$said" in
*"held by the legacy backend"*)
    ok "the collector answers degraded, and says the rules are in the old backend" ;;
*)
    bad "nftables is empty and the old backend holds the rules, and the agent did not say so"
    echo "$said" | head -c 900 ;;
esac
case "$said" in
*'"state":"degraded"'*) ok "and degraded is the state it reports, not unavailable" ;;
*)                      bad "the state the agent reports is not degraded: $(echo "$said" | grep -o '"name":"firewall"[^}]*' | head -c 200)" ;;
esac
case "$reading" in
*'"legacy_backend":true'*)
    ok "the reading itself carries the marker: legacy_backend is true" ;;
*)
    bad "the reading does not carry the legacy marker"; echo "$reading" | head -c 600 ;;
esac
case "$reading" in
*"fw-backend|legacy"*) ok "and a row of its own says which tables the old backend holds" ;;
*)                     bad "no fw-backend row in the reading" ;;
esac
if grep -q "firewall.disabled" /tmp/vigil-legacy/log 2>/dev/null; then
    bad "the agent called this host a host without a firewall: the worst outcome of the two"
else
    ok "and nothing anywhere calls this a host without a firewall"
fi
iptables-legacy -D INPUT -p tcp --dport 4444 -j DROP 2>/dev/null

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
