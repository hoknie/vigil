# vigild

> The daemon: it reads the host, compares, judges, keeps and reports — and answers the console.

## Why

`vigild` assembles the agent. It links the subject crates and asks each one for its collector, its
rules and its settings. What it owns is what a running process owns: the loaded configuration, the
identity of this host and installation, the clock, the watch round, the store, the reporters and
the socket.

It runs in one process on plain threads. A watch round is a few reads and a few writes, and the
agent is a guest on the host, so the product carries no async runtime.

## What's inside

| | |
|---|---|
| **Start-up** | reads `vigil.yaml` and every collector block, refuses what it cannot honour with a reason, says what it will watch |
| **Identity** | `host_id`, a keyed hash of the machine id, and a persisted `install_id`; a move to another machine is reported once |
| **The round** | each collector on its own schedule: read, compare, judge, suppress, keep, report |
| **The agent itself** | `agent.*` findings: baseline ready, collector failing or recovered, budget, buffer, store |
| **The socket** | a `0600` unix socket, `/run/vigil/vigil.sock` on Linux, with three questions and three verbs — `kill`, `change`, `control` — each off until its collector's block switches it on |
| **Commands** | `vigild configure`, `vigild collector <name> enable\|disable`, `vigild suppress` |

## Depends on

- `vigil-model` · `vigil-config` · `vigil-rules` · `vigil-collect` · `vigil-module`
- every crate under `crates/collectors/`
- `vigil-store` · `vigil-report`
- `clap`, `rustix`, `serde`, `serde_json`, `yaml_serde`, `sha2`

## Where it fits

```text
collectors/vigil-<subject> ──┐
vigil-config ────────────────┤
vigil-store ─────────────────┼──► vigild ──── unix socket ───► vigil
vigil-report ────────────────┘      │
                                    └──► NDJSON file · syslog
```
