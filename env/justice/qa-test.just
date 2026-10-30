set shell := ["bash", "-c"]
set working-directory := '../..'

import 'common.just'

[private]
default:
    @{{ JUST }} --list qa-test --list-heading $'just qa-test <what>\n' --list-prefix '    '

# Tests. They need no machine state, no network and no database; keep it that way
all:
    {{ CARGO }} test --workspace

# Write the samples again from what the code declares and the parsers build, then show what the console owes
golden:
    VIGIL_GOLDEN=write {{ CARGO }} test -p vigil-resources -p vigil-files -p vigild
    -{{ CARGO }} test -p vigil

# Test coverage, one line per file and a total; only the code this machine compiles
coverage:
    {{ CARGO }} llvm-cov --workspace --summary-only

# Test coverage as an HTML report, opened in the browser
coverage-html:
    {{ CARGO }} llvm-cov --workspace --html --open

# What a reading, a diff and a judgement cost on this machine
cost:
    {{ CARGO }} run --release --example network -p vigil-network
    {{ CARGO }} run --release --example users -p vigil-users
    {{ CARGO }} run --release --example firewall -p vigil-firewall
    {{ CARGO }} run --release --example containers -p vigil-containers
    {{ CARGO }} run --release --example engines -p vigil-engines
    {{ CARGO }} run --release --example persistence -p vigil-persistence
    {{ CARGO }} run --release --example processes -p vigil-processes
    {{ CARGO }} run --release --example launches -p vigil-launches
    {{ CARGO }} run --release --example resources -p vigil-resources
    {{ CARGO }} run --release --example files -p vigil-files
    {{ CARGO }} run --release --example store -p vigil-store
