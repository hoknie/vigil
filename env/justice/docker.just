# Build everything, start the daemon, open a shell next to it
[group('docker')]
docker:
    {{ RUN }} cargo build --workspace
    {{ COMPOSE }} up --detach vigild
    @if [ -z "$({{ COMPOSE }} ps --status running --quiet vigild)" ]; then \
        echo; echo "the vigild service is not running. What it said:"; echo; \
        {{ COMPOSE }} logs --tail 15 vigild; exit 1; \
    fi
    trap '{{ COMPOSE }} stop vigild' EXIT INT TERM; {{ COMPOSE }} run --rm shell bash -l

# Drive the console under a real terminal and check what it does
[group('docker')]
console-pty:
    {{ COMPOSE }} up --detach vigild
    {{ COMPOSE }} exec -T vigild bash -c 'cargo build --quiet --bin vigil --bin vigild && for _ in $(seq 1 300); do [ -S /run/vigil/vigil.sock ] && break; sleep 1; done && timeout -s KILL 420 expect -f env/docker/console-pty.exp /build/target/debug/vigil'

# What the daemon service is seeing, as it sees it
[group('docker')]
docker-logs:
    {{ COMPOSE }} logs --follow vigild

# Build the product, test it and start it, the same thing `docker compose up` does
[group('docker')]
docker-smoke:
    {{ RUN }} ./env/docker/smoke.sh

# The whole gate, on musl, inside the container
[group('docker')]
docker-check:
    {{ RUN }} just check

# The path from the kernel through a live auditd to a finding. Changes the HOST kernel's
# audit state, not the container's, and puts it back; refuses to run beside another auditd
[group('docker')]
docker-audit:
    {{ COMPOSE }} run --rm --build audit

# The daemon against CONFIG, in the foreground, inside the container (Ctrl-C to stop)
[group('docker')]
docker-daemon:
    {{ RUN }} just daemon

# The terminal console, inside the container (q or Esc to leave)
[group('docker')]
docker-console:
    {{ RUN }} just console

# Rebuild the image without the layer cache, after changing env/docker/Dockerfile
[group('docker')]
docker-build:
    {{ COMPOSE }} build --no-cache

# Remove the containers AND the volumes: build output, the crates.io cache, the agent's state
[group('docker')]
docker-clean:
    -{{ COMPOSE }} rm --force --stop --volumes
    {{ COMPOSE }} down --volumes --remove-orphans

# The same measurement, on musl, where every collector compiles
[group('docker')]
docker-cost:
    {{ RUN }} just cost
