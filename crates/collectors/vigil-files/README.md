# vigil-files

The files this host is expected to keep as they are: their content, their permissions and a new suid bit, for every path the watch list names.

## The decision it follows from

A recursive walk that hashes everything is the shortest path to an agent the customer
disables first. So only what the watch list names is read, under a ceiling of paths per reading
and of bytes per file, and a walk that reaches a ceiling says what it left unwalked rather than
pretending it saw everything.

## Settings

Its block is `files`, shipped in `config/collectors/files.yaml` with the defaults and a comment above
every key saying what it means.

## Dependencies

`vigil-model`, `vigil-collect` (the `Collector` port), `vigil-rules` (`RuleSet`), `vigil-module`, `vigil-view`, `serde`, `serde_json`, `yaml_serde` for the watch list; `libc` on macOS.

## Context

```
vigil-model · vigil-collect · vigil-rules · vigil-module · vigil-view
                                   ▲
                             vigil-files
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
             bin/vigild (reading, rules)   bin/vigil (its screen)
```
