# vigil

> The console: what the daemon on this host has read and found, in a terminal.

## Why

Everything the console shows comes from the daemon over the local socket, so an operator sees
exactly what the agent knows. The console keeps nothing of its own.

It draws what each subject describes: sections, subject screens and cards come from the subject
crates, in the order the modules are linked. The console adds what belongs to it — navigation,
search, history, the difference behind a finding, and the questions it asks before a verb.

It reads well with no colour and at eighty columns, and a daemon that is silent gets a screen of
its own that says so.

## What's inside

| | |
|---|---|
| **`vigil`** | the console: sections, summary, findings, the difference behind a finding, a screen per subject |
| **`vigil capture`** | one page rendered as text for a script; with `--once` it reports where it moved and exits with code 2 |
| **`vigil suppress`** | edits the suppressions in the configuration |

## Depends on

- `vigil-model` · `vigil-config` · `vigil-view` · `vigil-module`
- every crate under `crates/collectors/`, for its screen
- `ratatui`, `tui-input`, `clap`, `serde_json` — `vigil` is the one member that links `ratatui`

## Where it fits

```text
collectors/vigil-<subject> · views ──┐
vigil-view ──────────────────────────┼──► vigil ◄──── unix socket ──── vigild
vigil-config ────────────────────────┘
```
