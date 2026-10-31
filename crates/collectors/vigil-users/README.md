# vigil-users

Who may log in to this host, as whom, and with what: accounts, groups, sudo grants, keys, sessions.

## The decision it follows from

The accounts are read from the files the system keeps them in. The console may change an
account, a group, a sudo grant in `/etc/sudoers.d`, a key or a session — it edits and removes,
it never creates an account or a grant — only when the operator switches it on in this block.

## Settings

Its block is `users`, shipped in `config/collectors/users.yaml` with the defaults and a comment above
every key saying what it means.

## Dependencies

`vigil-model`, `vigil-collect` (the `Collector` port), `vigil-rules` (`RuleSet`), `vigil-module`, `vigil-view`, `serde_json`; `libc` on macOS.

## Context

```
vigil-model · vigil-collect · vigil-rules · vigil-module · vigil-view
                                   ▲
                             vigil-users
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
             bin/vigild (reading, rules)   bin/vigil (its screen)
```
