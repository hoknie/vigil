# vigil-launches

What people run on this host, from the kernel's audit records.

## The decision it follows from

A launch is read from what the kernel already records — on Linux the `vigil_exec` audit rule
in `/etc/audit/rules.d` and the auditd plugin in `/etc/audit/plugins.d/vigil.conf`, on macOS
`eslogger` run by the launchd job `vigil.launches` — and not by watching processes come and go,
which misses everything shorter than a reading. A row is one person and one program, frozen when
it first appears.

## Settings

Its block is `launches`, shipped in `config/collectors/launches.yaml` with the defaults and a comment above
every key saying what it means.

## Dependencies

`vigil-model`, `vigil-collect` (the `Collector` port), `vigil-rules` (`RuleSet`), `vigil-module`, `vigil-view`, `serde`, `serde_json`, `rustix`.

## Context

```
vigil-model · vigil-collect · vigil-rules · vigil-module · vigil-view
                                   ▲
                             vigil-launches
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
             bin/vigild (reading, rules)   bin/vigil (its screen)
```
