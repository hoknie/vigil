# vigil-engines

The images, volumes, networks, compose projects, pods and secrets the container engines of this host hold.

## The decision it follows from

The daemon starts no program of its own, and a docker or podman client is root on this host
in one step. So the engines are asked by `vigil-container-dump`, run by `vigil-containers.timer`
(on macOS, the launchd job `vigil.containers`), and this crate reads the documents it writes.
Each document names the moment of the one before it, and a reading is called stale by two of
the periods the documents show.

## Settings

Its block is `containers-engines`, shipped in `config/collectors/containers.yaml` with the defaults and a comment above
every key saying what it means.

## Dependencies

`vigil-model`, `vigil-collect` (the `Collector` port), `vigil-rules` (`RuleSet`), `vigil-module`, `vigil-view`, `serde`, `serde_json`. The dump program links this crate for the document it writes.

## Context

```
vigil-model · vigil-collect · vigil-rules · vigil-module · vigil-view
                                   ▲
                             vigil-engines
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
             bin/vigild (reading, rules)   bin/vigil (its screen)
```
