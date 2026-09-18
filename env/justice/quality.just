# Tests. They need no machine state, no network and no database; keep it that way
[group('quality')]
test:
    {{ CARGO }} test --workspace

# Lint, warnings are errors: exactly what CI runs
[group('quality')]
clippy:
    {{ CARGO }} clippy --workspace --all-targets -- -D warnings

# Format the code
[group('quality')]
fmt:
    {{ CARGO }} fmt

# Check formatting (no writes)
[group('quality')]
fmt-check:
    {{ CARGO }} fmt --check

# The whole gate, in CI's order
[group('quality')]
check: fmt-check clippy test

# What a reading, a diff and a judgement cost on this machine
[group('quality')]
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

# Write the samples again from what the code declares and the parsers build, then show what the console owes
[group('quality')]
golden:
    VIGIL_GOLDEN=write {{ CARGO }} test -p vigil-resources -p vigil-files -p vigild
    -{{ CARGO }} test -p vigil
