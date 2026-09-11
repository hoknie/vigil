#!/usr/bin/env bash

set -uo pipefail

cd "$(dirname "$0")/../.."

RULESET="${RULESET:-/var/lib/vigil/firewall/ruleset.json}"
PLAN=/tmp/vigil-firewall-cost.nft

say() { printf '\n\033[1m== %s\033[0m\n' "$*"; }

if ! nft list ruleset >/dev/null 2>&1; then
    echo "no nft here: this measurement needs --cap-add NET_ADMIN and the nftables package" >&2
    exit 2
fi

take_the_reading() {
    mkdir -p "$(dirname "$RULESET")"
    nft --json list ruleset > "$RULESET"
    chmod 0600 "$RULESET"
}

build() {
    local bans="$1" published="$2"
    {
        echo 'flush ruleset'
        echo 'table ip nat {'
        echo '  chain PREROUTING { type nat hook prerouting priority -100; policy accept; }'
        echo '  chain POSTROUTING { type nat hook postrouting priority 100; policy accept; }'
        echo '  chain DOCKER {'
        for port in $(seq 1 "$published"); do
            echo "    tcp dport $((30000 + port)) counter accept"
        done
        echo '  }'
        echo '}'
        echo 'table ip filter {'
        echo '  chain FORWARD { type filter hook forward priority 0; policy drop; }'
        echo '  chain DOCKER-USER { }'
        echo '  chain DOCKER {'
        for port in $(seq 1 "$published"); do
            echo "    tcp dport $((30000 + port)) counter accept"
        done
        echo '  }'
        echo '}'
        echo 'table inet filter {'
        echo '  chain input { type filter hook input priority 0; policy drop;'
        echo '    iif lo accept'
        echo '    ct state established,related accept'
        echo '    tcp dport 22 accept'
        echo '  }'
        echo '  chain forward { type filter hook forward priority 0; policy drop; }'
        echo '  chain output { type filter hook output priority 0; policy accept; }'
        echo '}'
        echo 'table inet f2b-table {'
        echo '  chain f2b-chain { type filter hook input priority -1; policy accept;'
        for ban in $(seq 1 "$bans"); do
            echo "    ip saddr 203.0.113.$((ban % 254 + 1)) tcp dport $((1024 + ban)) counter drop"
        done
        echo '  }'
        echo '}'
    } > "$PLAN"

    nft -f "$PLAN" || { echo "nft would not take the plan in $PLAN" >&2; exit 1; }
    take_the_reading
}

measure() {
    echo "  $1: $(wc -c < "$RULESET") bytes of nft --json, $(grep -o '"rule":' "$RULESET" | wc -l | tr -d ' ') rule(s) in it"
    cargo run --release --quiet --example cost -p vigil-collect -- firewall "${ROUNDS:-2000}"
}

say "a host with nothing in its ruleset"
nft flush ruleset
take_the_reading
measure "empty"

say "a host running docker: published ports, a nat table and a filter table"
build 0 40
measure "docker"

say "a host running docker and fail2ban with a few hundred addresses banned"
build 640 40
measure "docker and fail2ban"

say "the ruleset is left flushed behind this"
nft flush ruleset
take_the_reading
