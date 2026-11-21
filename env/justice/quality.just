# The gate, in CI's order: formatting, lint, tests; quick is what a push runs, full is what the night runs
[group('quality')]
qa-gates set='quick':
    @case '{{ set }}' in quick|full) ;; *) echo "qa-gates: the set is quick or full, not {{ set }}" >&2; exit 2 ;; esac
    {{ JUST }} qa-lint fmt-check
    {{ JUST }} qa-lint clippy
    @if [ '{{ set }}' = full ]; then {{ JUST }} qa-test all; else {{ JUST }} qa-test quick; fi
