# vigil-firewall

> The rules by which this host lets network traffic in and drops it.

## Why

Reading the ruleset takes `nft` on Linux and `pfctl` on macOS, run as root. The daemon starts
no program of its own, so a separate job writes the ruleset to a file: `vigil-firewall.timer` on
Linux, the launchd job `vigil.firewall` on macOS. This crate reads that file.

## What's inside

| | |
|---|---|
| **Reads** | the ruleset file written by the firewall job |
| **Raises** | `firewall.*` |
| **Every** | 60 seconds |
| **Settings** | the `firewall` block, shipped in `config/collectors/firewall.yaml` with a comment above every key |

## Depends on

- `vigil-model` · `vigil-collect` · `vigil-rules` · `vigil-module` · `vigil-view`
- `serde`, `serde_json`; `libc` on macOS

## Where it fits

```text
vigil-model   vigil-collect   vigil-rules   vigil-module   vigil-view
     └──────────────┴──────────────┼──────────────┴──────────────┘
                                   ▼
                             vigil-firewall
                                   │
                   ┌───────────────┴───────────────┐
                   ▼                               ▼
          vigild · reads and judges        vigil · draws its screen
```
