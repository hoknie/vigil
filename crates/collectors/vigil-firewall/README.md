# vigil-firewall

The rules by which this host lets network in and drops it.

## The decision it follows from

Reading the ruleset needs `nft` (on macOS, `pfctl`) run as root, and the daemon starts no program
of its own. So `vigil-firewall.timer` (on macOS, the launchd job `vigil.firewall`) writes the
ruleset to a file, and this crate reads that file.

## Settings

Its block is `firewall`, shipped in `config/collectors/firewall.yaml` with the defaults and a comment above
every key saying what it means.

## Dependencies

`vigil-model`, `vigil-collect` (the `Collector` port), `vigil-rules` (`RuleSet`), `vigil-module`, `vigil-view`, `serde`, `serde_json`; `libc` on macOS.

## Context

```
vigil-model · vigil-collect · vigil-rules · vigil-module · vigil-view
                                   ▲
                             vigil-firewall
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
             bin/vigild (reading, rules)   bin/vigil (its screen)
```
