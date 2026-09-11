# API docs for this workspace only
[group('docs')]
doc:
    {{ CARGO }} doc --workspace --no-deps --document-private-items
