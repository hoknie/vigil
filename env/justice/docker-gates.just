set shell := ["bash", "-c"]
set working-directory := '../..'

import 'common.just'

[private]
default:
    @{{ JUST }} --list docker-gates --list-heading $'just docker-gates <what>\n' --list-prefix '    '

# The whole gate, on musl, inside the container
all:
    {{ RUN }} just qa-gates

# Build the product, test it and start it, the same thing `docker compose up` does
smoke:
    {{ RUN }} ./env/docker/smoke.sh

# The path from a live auditd to a finding; changes the HOST kernel's audit state and puts it back
audit:
    {{ COMPOSE }} run --rm --build audit

# Drive the console under a real terminal and check what it does
console-pty:
    {{ COMPOSE }} up --detach vigild
    {{ COMPOSE }} exec -T vigild bash -c 'cargo build --quiet --bin vigil --bin vigild && for _ in $(seq 1 300); do [ -S /run/vigil/vigil.sock ] && break; sleep 1; done && timeout -s KILL 420 expect -f env/docker/console-pty.exp /build/target/debug/vigil'
