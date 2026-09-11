# Remove build artifacts
[group('housekeeping')]
clean:
    {{ CARGO }} clean
