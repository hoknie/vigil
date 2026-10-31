# vigil-rules

> How the difference between two readings becomes findings.

## Why

Every rule is written over snapshots. A collector reads the host into a `Snapshot`, `diff` compares
it with the previous one, and each `Rule` looks at a change and says what it means. Every source of
facts — `/proc`, a log, a spool of audit records — is turned into a snapshot first, so there is one
path from the host to a finding and one place where each thing is detected.

The crate has no io, no clock and no async. A rule is tested with two snapshots built in the test.

## What's inside

| | |
|---|---|
| **`diff`** | the changes between two readings of one collector: added, removed, changed, row by row |
| **`Rule`, `BatchRule`** | the ports a subject implements — one change at a time, or all the changes of a round together |
| **`RuleContext`** | the time of the round and the way to mint an event id |
| **`RuleSet`** | the rules of one subject, run together over the changes of a round |
| **`findings_for`, `programs_for`** | what a set of changes comes to, for a subject's tests |
| **helpers** | whether a path belongs to this host, whether anyone may write to it |
| **`fixture`** | snapshots and contexts for the tests of the subject crates |

## Depends on

- `vigil-model`
- `serde_json`

## Where it fits

```text
vigil-model ──► vigil-rules ──► collectors/vigil-<subject>/rules
                     │
                     └────────► vigild · runs each subject's RuleSet every round
```
