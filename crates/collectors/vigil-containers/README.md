# vigil-containers

The containers running on this host, what they may do and what of this host they hold.

## The decision it follows from

A container is read from `/proc` — cgroups, capabilities and mount tables — not from an
engine's socket, so the agent learns what runs here without talking to anything that could
start a container. The sockets of the container runtimes are read as files, by mode and owner:
a socket anyone may write to is the host handed over. One list of sockets serves the reading
and the rule, so a socket the rule calls dangerous is always read, and one socket reached by
two paths is one row.

## Settings

Its block is `containers`, shipped in `config/collectors/containers.yaml` with the defaults and a comment above
every key saying what it means.

## Dependencies

`vigil-model`, `vigil-collect` (the `Collector` port), `vigil-rules` (`RuleSet`), `vigil-module`, `vigil-view`, `serde_json`. No io of its own outside `collectors/`.

## Context

```
vigil-model · vigil-collect · vigil-rules · vigil-module · vigil-view
                                   ▲
                             vigil-containers
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
             bin/vigild (reading, rules)   bin/vigil (its screen)
```
