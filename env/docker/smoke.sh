#!/usr/bin/env bash

set -uo pipefail
cd "$(dirname "$0")/../.."

say() { printf '\n\033[1m== %s\033[0m\n' "$*"; }

CONFIG=/tmp/vigil-smoke.yaml
FINDINGS=/tmp/vigil-smoke-findings.ndjson
PORT=4444

cat > "$CONFIG" <<YAML
state_dir: /tmp/vigil-smoke-state
socket_path: /tmp/vigil-smoke.sock
interval_seconds: 3
reporters:
  - kind: ndjson
    path: $FINDINGS
YAML
rm -f "$FINDINGS"

say "toolchain"
rustc --version
echo "host triple: $(rustc -vV | awk '/^host:/{print $2}')   # musl — the release target's libc"

say "build (workspace, debug)"
cargo build --workspace || exit 1

say "tests"
cargo test --workspace --quiet || exit 1

say "the agent, watching this container"
"${CARGO_TARGET_DIR:-target}/debug/vigild" "$CONFIG" &
daemon=$!
trap 'kill "$daemon" 2>/dev/null; kill "$listener" 2>/dev/null' EXIT

sleep 5
if ! kill -0 "$daemon" 2>/dev/null; then
    echo "the daemon stopped on its own: that is the failure this check exists to catch" >&2
    exit 1
fi

say "opening a listening port on :$PORT"
nc -l -p "$PORT" >/dev/null 2>&1 &
listener=$!
sleep 8

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

say "result"
echo "Built on musl, tests green, and the agent noticed a port opened while it watched:"
echo "collector → differ → rule → reporter, end to end."
echo
echo "The findings, as a receiver would have been sent them: $FINDINGS"
echo "For a shell in here, with the daemon running next door:  just docker"
