# The whole gate, in CI's order: formatting, lint, tests
[group('quality')]
qa-gates:
    {{ JUST }} qa-lint fmt-check
    {{ JUST }} qa-lint clippy
    {{ JUST }} qa-test all
