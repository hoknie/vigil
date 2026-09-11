# Debug build of the whole workspace
[group('build')]
build:
    {{ CARGO }} build --workspace

# Release build of the whole workspace
[group('build')]
release:
    {{ CARGO }} build --release --workspace
