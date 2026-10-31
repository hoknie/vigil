# vigil-launches

> What people run on this host, from the kernel's audit records.

## Why

Every launch is already recorded by the kernel. On Linux the `vigil_exec` audit rule and the
auditd plugin `vigil-audit-plugin` deliver the records; on macOS `vigil-launches-spool` runs
`eslogger`. Both write a spool, and this crate reads it, so even a program that lives for a
millisecond is seen.

A row is one person and one program, recorded when it first appears.

## What's inside

| | |
|---|---|
| **Reads** | the spool of exec records |
| **Raises** | `exec.first_seen_for_user`, `exec.from_writable_path` |
| **Every** | 15 seconds |
| **Settings** | the `launches` block, shipped in `config/collectors/launches.yaml` with a comment above every key |

## Depends on

- `vigil-model` · `vigil-collect` · `vigil-rules` · `vigil-module` · `vigil-view`
- `serde`, `serde_json`, `rustix`

## Where it fits

```text
vigil-model   vigil-collect   vigil-rules   vigil-module   vigil-view
     └──────────────┴──────────────┼──────────────┴──────────────┘
                                   ▼
                             vigil-launches
                                   │
                   ┌───────────────┴───────────────┐
                   ▼                               ▼
          vigild · reads and judges        vigil · draws its screen
```
