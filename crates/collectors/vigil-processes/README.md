# vigil-processes

> The programs running on this host.

## Why

Processes are read from the kernel. Secrets in a command line — `mysql -p…`, an
`Authorization` header — are hidden on the agent before anything is written to disk.

The console can close a listening socket or stop a program a person marked, with SIGTERM or
SIGKILL. That ability is off until the operator switches it on in this block, and every attempt
is itself a finding.

On a Mac run by an ordinary account, the command lines of other accounts' processes are kept by
macOS for root. The collector names that as a limit of the mode, and a reading that went wrong
stays `degraded`.

## What's inside

| | |
|---|---|
| **Reads** | the process table: program, arguments, owner, parent |
| **Raises** | `process.*` |
| **Every** | 30 seconds |
| **Settings** | the `processes` block, shipped in `config/collectors/processes.yaml` with a comment above every key |

## Depends on

- `vigil-model` · `vigil-collect` · `vigil-rules` · `vigil-module` · `vigil-view`
- `serde_json`; `libc` on macOS

## Where it fits

```text
vigil-model   vigil-collect   vigil-rules   vigil-module   vigil-view
     └──────────────┴──────────────┼──────────────┴──────────────┘
                                   ▼
                            vigil-processes
                                   │
                   ┌───────────────┴───────────────┐
                   ▼                               ▼
          vigild · reads and judges        vigil · draws its screen
```
