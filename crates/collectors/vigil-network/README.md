# vigil-network

What this host listens on, and who holds each socket: the reading, the rules over it, and the
screen that shows it.

## The decision it follows from

One subject is one crate (`docs/designs/2026-09-12-DESIGN-modules.md`). Before this crate
existed, "what vigil knows about listening sockets" was six folders in four crates, and adding
the seventh meant editing four tables that nothing checks.

## What is here

- `parsers/` — `/proc/net/{tcp,tcp6,udp,udp6}` and `/proc/net/unix`, plus the reading they
  build. Portable and tested against fixtures: no io in this folder;
- `collectors/linux/` — the io half: reading those files and walking
  `inode → /proc/*/fd → pid → exe` for the process behind each socket. `Health::Degraded` when
  the owners cannot be resolved, because a socket with no owner is not a socket with no
  attacker;
- `rules/` — the difference between two readings: a new socket, one on a writable path or a
  deleted binary, an owner that changed, a socket that closed, and the batch rule that says how
  much of this host is exposed;
- `fixture/` — one sample reading built by the real parsers (the golden files in
  `vigil-model/golden/` are written from it) and the single rows the rules are tested on;
- `modules/` — the declaration: the name `ports`, the period of 30 seconds, the finding family
  `port.listen`, and how to build the collector and the rule set from the settings.

## Dependencies

`vigil-model` for the vocabulary, `vigil-collect` for the `Collector` port and the shared
readers (`redact`, `parse_passwd`), `vigil-rules` for the `Rule` ports and `RuleSet`,
`vigil-module` for the declaration. No neighbour module, and no `ratatui`.

## Context

```
/proc/net/* ──► parsers/ ──► collectors/linux/ ──► Snapshot ──► rules/ ──► Finding
                                   ▲                                │
                              modules/ports.rs ◄───────────────────┘
                                   │
                     bin/vigild (reads and judges) · bin/vigil (draws)
```
