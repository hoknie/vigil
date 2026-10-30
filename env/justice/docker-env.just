set shell := ["bash", "-c"]
set working-directory := '../..'

import 'common.just'

[private]
default:
    @{{ JUST }} --list docker-env --list-heading $'just docker-env <what>\n' --list-prefix '    '

# Build everything, start the daemon, open a shell next to it
shell:
    {{ RUN }} cargo build --workspace
    {{ COMPOSE }} up --detach vigild
    @if [ -z "$({{ COMPOSE }} ps --status running --quiet vigild)" ]; then \
        echo; echo "the vigild service is not running. What it said:"; echo; \
        {{ COMPOSE }} logs --tail 15 vigild; exit 1; \
    fi
    trap '{{ COMPOSE }} stop vigild' EXIT INT TERM; {{ COMPOSE }} run --rm shell bash -l

# Remove the containers AND the volumes: build output, the crates.io cache, the agent's state
clean:
    -{{ COMPOSE }} rm --force --stop --volumes
    {{ COMPOSE }} down --volumes --remove-orphans
