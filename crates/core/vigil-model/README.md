# vigil-model

> The shared vocabulary: findings, snapshots, the host, and the format they travel in.

## Why

`host-findings/v1` is where this product meets the outside world, so it is written down once, as
types. The daemon builds a `Finding`, the store writes it, a reporter sends it and the console
shows it — all with the same type.

The local protocol between the console and the daemon lives here too. The console links this
crate and has everything it needs to talk to the daemon.

The crate does no io on the watch path. `Golden` reads and writes the recorded samples under
`golden/` for tests, and nothing a running agent does reaches it.

## What's inside

| | |
|---|---|
| **Contract** | `Envelope`, `Producer`, `SchemaVersion` and `SCHEMA_VERSION` — a receiver of version N accepts N and N−1. `docs/contract/` holds the same contract as JSON Schema |
| **Findings** | `Finding`, `Kind` with the reserved kinds in `RESERVED`, `Severity`, `State`, `Subject`, `Evidence` |
| **Host** | `Host`, `Os`, `Peer` |
| **Snapshots** | `Snapshot` keyed by row, `Change` between two of them; `Golden`, `Shape`, `Settled` for recorded samples |
| **Local protocol** | `Request`, `Response`, the status of the agent and its parts, the reports of `kill`, `change` and `control` |
| **Forms** | `Rfc3339` and `Uuid7`, the two string forms the format fixes |

## Depends on

- `serde`, `serde_json`

## Where it fits

```text
                              vigil-model
                                   ▲
     ┌─────────────┬───────────────┼───────────────┬──────────────┐
vigil-collect  vigil-rules     vigil-view      vigil-store   vigil-report
                                   ▲
                       collectors/vigil-<subject>
                                   ▲
                        vigild ────┴──── vigil
```
