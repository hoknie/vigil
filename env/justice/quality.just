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
    {{ CARGO }} run --release --example cost -p vigil-rules
    for name in ports users persistence processes firewall launches; do \
        {{ CARGO }} run --release --example cost -p vigil-collect -- "$name"; \
    done

# Write the samples again from what the code declares and the parsers build, then show what the console owes
[group('quality')]
golden:
    VIGIL_GOLDEN=write {{ CARGO }} test -p vigil-collect -p vigild
    -{{ CARGO }} test -p vigil
