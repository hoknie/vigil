# vigil-resources

The boot this host is running, the moment it started, its clock, how busy its processors are,
how much of its memory is available and the room left on its filesystems.

## The decision it follows from

A host that runs out of room, reboots unasked or has its clock stepped is a host whose other
readings stop meaning what they meant.

## Settings

Its block is `resources`, shipped in `config/collectors/resources.yaml` with the defaults and a comment above
every key saying what it means.

## Dependencies

`vigil-model`, `vigil-collect` (the `Collector` port), `vigil-rules` (`RuleSet`), `vigil-module`, `vigil-view`, `serde`, `serde_json`.

## Context

```
vigil-model · vigil-collect · vigil-rules · vigil-module · vigil-view
                                   ▲
                             vigil-resources
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
             bin/vigild (reading, rules)   bin/vigil (its screen)
```
