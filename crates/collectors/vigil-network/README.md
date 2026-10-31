# vigil-network

The sockets this host listens on, and the process holding each one.

## The decision it follows from

A listening socket is a door into this host, and a door that appears is a finding. Sockets are
read from the kernel's own tables (`/proc/net` on Linux), and the program holding each one is
named, so a program that keeps moving its ports can be silenced by name rather than by port.

## Settings

Its block is `network`, shipped in `config/collectors/network.yaml` with the defaults and a comment above
every key saying what it means.

## Dependencies

`vigil-model`, `vigil-collect` (the `Collector` port), `vigil-rules` (`RuleSet`), `vigil-module`, `vigil-view`, `serde_json`; `libc` on macOS.

## Context

```
vigil-model · vigil-collect · vigil-rules · vigil-module · vigil-view
                                   ▲
                             vigil-network
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
             bin/vigild (reading, rules)   bin/vigil (its screen)
```
