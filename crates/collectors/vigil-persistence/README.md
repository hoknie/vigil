# vigil-persistence

> Everything this host starts by itself.

## Why

Persistence is how an intruder survives a reboot. The agent reads every way this host starts
something on its own — systemd units and timers, cron, kernel modules, `ld.so.preload`, shell
profiles, launchd jobs — and compares each reading with the one before.

From the console an operator can stop, start, disable, enable, mask and unmask a unit or a timer,
and comment a cron line out and back in. That ability is off until switched on in this block.

## What's inside

| | |
|---|---|
| **Reads** | units, timers, cron, kernel modules, preload, shell profiles, launchd jobs |
| **Raises** | `persistence.*` |
| **Every** | 300 seconds |
| **Settings** | the `persistence` block, shipped in `config/collectors/persistence.yaml` with a comment above every key |

## Depends on

- `vigil-model` · `vigil-collect` · `vigil-rules` · `vigil-module` · `vigil-view`
- `serde_json`

## Where it fits

```text
vigil-model   vigil-collect   vigil-rules   vigil-module   vigil-view
     └──────────────┴──────────────┼──────────────┴──────────────┘
                                   ▼
                           vigil-persistence
                                   │
                   ┌───────────────┴───────────────┐
                   ▼                               ▼
          vigild · reads and judges        vigil · draws its screen
```
