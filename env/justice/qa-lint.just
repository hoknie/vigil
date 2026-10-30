set shell := ["bash", "-c"]
set working-directory := '../..'

import 'common.just'

[private]
default:
    @{{ JUST }} --list qa-lint --list-heading $'just qa-lint <what>\n' --list-prefix '    '

# Format the code
fmt:
    {{ CARGO }} fmt

# Check formatting (no writes)
fmt-check:
    {{ CARGO }} fmt --check

# Lint, warnings are errors: exactly what CI runs
clippy:
    {{ CARGO }} clippy --workspace --all-targets -- -D warnings
