# vigil-persistence

What this host starts by itself: units, timers, cron, kernel modules, shell profiles and launchd jobs.

## The decision it follows from

Persistence is how an intruder outlives a reboot, so everything the host starts by itself is
read and compared with the reading before. The console may stop, start, disable, enable, mask
and unmask a unit or a timer, and comment a cron line out and back in — six words and their
opposites and nothing else — only when the operator switches it on in this block.

## Settings

Its block is `persistence`, shipped in `config/collectors/persistence.yaml` with the defaults and a comment above
every key saying what it means.

## Dependencies

`vigil-model`, `vigil-collect` (the `Collector` port), `vigil-rules` (`RuleSet`), `vigil-module`, `vigil-view`, `serde_json`.

## Context

```
vigil-model · vigil-collect · vigil-rules · vigil-module · vigil-view
                                   ▲
                             vigil-persistence
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
             bin/vigild (reading, rules)   bin/vigil (its screen)
```
