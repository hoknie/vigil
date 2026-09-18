#!/usr/bin/env bash

set -uo pipefail
cd "$(dirname "$0")/../.."

say() { printf '\n\033[1m== %s\033[0m\n' "$*"; }

CONFIG=/tmp/vigil-smoke.yaml
FINDINGS=/tmp/vigil-smoke-findings.ndjson
SOCKET=/tmp/vigil-smoke.sock
RULESET=/var/lib/vigil/firewall/ruleset.json
PORT=4444

SHIPPED="$(SCHEDULE=3 ./env/scripts/config-here.sh /tmp/vigil-smoke-config)"
cat > "$CONFIG" <<YAML
state_dir: /tmp/vigil-smoke-state
socket_path: $SOCKET
collectors_path: $(dirname "$SHIPPED")/collectors
reporters:
  - kind: ndjson
    path: $FINDINGS
YAML
rm -f "$FINDINGS"

daemon=0
listener=0

ask() { printf '%s\n' "$1" | nc -U -q 2 "$SOCKET" 2>/dev/null; }

read_the_ruleset() {
    mkdir -p "$(dirname "$RULESET")"
    nft --json list ruleset > "$RULESET"
    chmod 0600 "$RULESET"
}

waiting_for() {
    for _ in $(seq 1 20); do
        grep -q "$1" "$FINDINGS" 2>/dev/null && return 0
        sleep 2
    done
    return 1
}

say "toolchain"
rustc --version
echo "host triple: $(rustc -vV | awk '/^host:/{print $2}')   # musl — the release target's libc"

say "build (workspace, debug)"
cargo build --workspace || exit 1

say "tests"
cargo test --workspace --quiet || exit 1

if ! nft list ruleset >/dev/null 2>&1; then
    echo "no nft here: run this with --cap-add NET_ADMIN, or the firewall half proves nothing" >&2
    exit 1
fi

say "a ruleset this container really has: nft builds it, a file carries it to the agent"
nft --version
nft flush ruleset
nft add table inet filter
nft add chain inet filter input '{ type filter hook input priority 0 ; policy drop ; }'
nft add chain inet filter forward '{ type filter hook forward priority 0 ; policy drop ; }'
nft add rule inet filter input iif lo accept
nft add rule inet filter input tcp dport 22 accept
read_the_ruleset
echo "  $(wc -c < "$RULESET") bytes of nft --json in $RULESET"

say "the agent, watching this container"
"${CARGO_TARGET_DIR:-target}/debug/vigild" "$CONFIG" &
daemon=$!
trap 'kill "$daemon" 2>/dev/null; kill "$listener" 2>/dev/null' EXIT

sleep 5
if ! kill -0 "$daemon" 2>/dev/null; then
    echo "the daemon stopped on its own: that is the failure this check exists to catch" >&2
    exit 1
fi

say "the live ruleset is what the agent took for a baseline"
if ask '{"query":"snapshot","collector":"firewall"}' | grep -q 'fw-chain|inet filter|input'; then
    echo "  the live ruleset reached the agent, chain and all"
else
    echo "the agent read no chain out of the ruleset this container is running" >&2
    echo "what is in the file now:" >&2
    head -c 400 "$RULESET" >&2
    echo >&2
    echo "what the agent answered:" >&2
    ask '{"query":"snapshot","collector":"firewall"}' >&2
    exit 1
fi

say "opening a listening port on :$PORT"
nc -l -p "$PORT" >/dev/null 2>&1 &
listener=$!
sleep 8

say "the input policy goes from drop to accept, the way a mistake does"
nft chain inet filter input '{ type filter hook input priority 0 ; policy accept ; }'
read_the_ruleset
if waiting_for "firewall.policy_weakened"; then
    echo "  $(grep -o '"title":"[^"]*"' "$FINDINGS" | tail -1)"
else
    echo "the policy was weakened on this host and the agent said nothing about it" >&2
    exit 1
fi

say "the firewall section, as a script is given it: eighty columns, no colour"
NO_COLOR=1 "${CARGO_TARGET_DIR:-target}/debug/vigil" capture --screen firewall --socket "$SOCKET" || true

say "nft flush ruleset, the way an incident does"
nft flush ruleset
read_the_ruleset
if waiting_for "firewall.ruleset_flushed"; then
    echo "  $(grep -o '"kind":"firewall[^"]*"' "$FINDINGS" | tail -2 | tr '\n' ' ')"
else
    echo "the ruleset was flushed on this host and the agent said nothing about it" >&2
    exit 1
fi
if ! waiting_for "firewall.disabled"; then
    echo "the host stopped filtering and the agent did not say so" >&2
    exit 1
fi

say "the same section once the ruleset is gone: a reading, and it says what it means"
NO_COLOR=1 "${CARGO_TARGET_DIR:-target}/debug/vigil" capture --screen firewall --socket "$SOCKET" || true

say "what the agent reported"
kill "$daemon" 2>/dev/null
wait "$daemon" 2>/dev/null
kill "$listener" 2>/dev/null

if [ ! -s "$FINDINGS" ]; then
    echo "nothing was reported: a port was opened and the agent said nothing about it" >&2
    exit 1
fi

grep -o '"finding_key":"[^"]*"\|"kind":"[^"]*"\|"title":"[^"]*"' "$FINDINGS" | paste - - - 2>/dev/null || cat "$FINDINGS"

if ! grep -q "port.listen|tcp|0.0.0.0:$PORT" "$FINDINGS"; then
    echo >&2
    echo "the port on :$PORT is not in the findings: the chain is broken somewhere between" >&2
    echo "/proc/net/tcp and the reporter" >&2
    exit 1
fi

for kind in firewall.policy_weakened firewall.ruleset_flushed firewall.disabled; do
    if ! grep -q "$kind" "$FINDINGS"; then
        echo "$kind never reached the reporter" >&2
        exit 1
    fi
done

say "result"
echo "Built on musl, tests green, and the agent noticed a port opened while it watched"
echo "and a ruleset weakened and then flushed under a live nft:"
echo "collector → differ → rule → reporter, end to end, twice over."
echo
echo "The findings, as a receiver would have been sent them: $FINDINGS"
echo "For a shell in here, with the daemon running next door:  just docker"
