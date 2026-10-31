# vigil-processes

The programs running on this host.

## The decision it follows from

The processes are read from the kernel, and secrets in a command line are redacted on the
agent before anything is written. The console may close a listening socket or stop a program a
person marked — SIGTERM, SIGKILL or closing the socket itself — only when the operator switches
it on in this block.

## Settings

Its block is `processes`, shipped in `config/collectors/processes.yaml` with the defaults and a comment above
every key saying what it means.

## Dependencies

`vigil-model`, `vigil-collect` (the `Collector` port), `vigil-rules` (`RuleSet`), `vigil-module`, `vigil-view`, `serde_json`; `libc` on macOS.

## Context

```
vigil-model · vigil-collect · vigil-rules · vigil-module · vigil-view
                                   ▲
                             vigil-processes
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
             bin/vigild (reading, rules)   bin/vigil (its screen)
```
