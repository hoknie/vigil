#!/usr/bin/env bash

set -uo pipefail
cd "$(dirname "$0")/../.."

say() { printf '\n\033[1m== %s\033[0m\n' "$*"; }

CONFIG=/tmp/vigil-kill-smoke.yaml
SOCKET=/tmp/vigil-kill-smoke.sock
STATE=/tmp/vigil-kill-smoke-state
PORT=4444

rm -rf "$STATE" "$SOCKET"
mkdir -p "$STATE"
cat > "$CONFIG" <<YAML
state_dir: $STATE
socket_path: $SOCKET
collectors: [ports]
schedule:
  ports: 2
killing:
  from_the_console: true
reporters: []
YAML

daemon=0
listener=0
failures=0

ask() { printf '%s\n' "$1" | nc -U -q 2 "$SOCKET" 2>/dev/null; }

wants() {
    if printf '%s' "$2" | grep -q -- "$3"; then
        printf '  ok   %s\n' "$1"
    else
        printf '  BAD  %s\n     wanted %s in: %s\n' "$1" "$3" "$2"
        failures=$((failures + 1))
    fi
}

cleanup() {
    [ "$daemon" -ne 0 ] && kill "$daemon" 2>/dev/null
    [ "$listener" -ne 0 ] && kill "$listener" 2>/dev/null
    return 0
}
trap cleanup EXIT

say "build"
cargo build --bin vigild || exit 1
VIGILD=$(command -v vigild || echo /build/target/debug/vigild)

say "a process this container really runs, listening on $PORT"
nc -l -p "$PORT" >/dev/null 2>&1 &
listener=$!
sleep 1
kill -0 "$listener" || { echo "the listener did not start" >&2; exit 1; }

"$VIGILD" "$CONFIG" > /tmp/vigil-kill-smoke.log 2>&1 &
daemon=$!
sleep 6

say "the agent read the pid holding it"
reading=$(ask '{"query":"snapshot","collector":"ports"}')
wants "the socket is in the reading" "$reading" "tcp|0.0.0.0:$PORT"
wants "and it carries the pid" "$reading" "\"pid\":$listener"

say "the console asks the agent to close it"
answer=$(ask "{\"query\":\"kill\",\"sockets\":[\"tcp|0.0.0.0:$PORT\"],\"killing\":\"terminate\"}")
wants "the agent answers what it did" "$answer" '"done":true'
wants "and names the process it did it to" "$answer" "\"pid\":$listener"

sleep 1
if kill -0 "$listener" 2>/dev/null; then
    printf '  BAD  the process is still running\n'
    failures=$((failures + 1))
else
    printf '  ok   the process is gone\n'
    listener=0
fi

say "what it refuses, it refuses by name"
gone=$(ask '{"query":"kill","sockets":["tcp|0.0.0.0:9"],"killing":"terminate"}')
wants "a socket not in the reading" "$gone" "no longer in the reading"

say "and every attempt is a finding of its own"
sleep 4
raised=$(ask '{"query":"findings"}')
wants "the one it carried out" "$raised" "agent.socket.killed"
wants "the one it refused" "$raised" "agent.socket.kill_refused"
wants "and the reading afterwards says the port closed" "$raised" "port.listen.removed"

say "the daemon said all of it out loud"
tail -14 /tmp/vigil-kill-smoke.log

if [ "$failures" -ne 0 ]; then
    printf '\n%d check(s) failed\n' "$failures" >&2
    exit 1
fi
printf '\nevery check passed\n'
