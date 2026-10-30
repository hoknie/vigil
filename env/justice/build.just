set shell := ["bash", "-c"]
set working-directory := '../..'

import 'common.just'

[private]
default:
    @{{ JUST }} --list build --list-heading $'just build <what>\n' --list-prefix '    '

# Debug build of the whole workspace
all:
    {{ CARGO }} build --workspace

# Release build of the whole workspace
release:
    {{ CARGO }} build --release --workspace

# API docs for this workspace only
docs:
    {{ CARGO }} doc --workspace --no-deps --document-private-items

# Remove build artifacts
clean:
    {{ CARGO }} clean
