# vigil-resources

> The boot, the clock, the processors, the memory and the room left on the disks.

## Why

A host that runs out of room, reboots unasked or has its clock stepped is a host whose other
readings lose their meaning. The agent watches the ground the other readings stand on.

Processor busyness is averaged over 15 minutes in steps of 10 %, available memory in steps of
5 %, so a finding marks a threshold crossed.

## What's inside

| | |
|---|---|
| **Reads** | boot id and time, clock, processor busyness, available memory, filesystems |
| **Raises** | `resource.*` |
| **Every** | 60 seconds |
| **Settings** | the `resources` block, shipped in `config/collectors/resources.yaml` with a comment above every key |

## Depends on

- `vigil-model` · `vigil-collect` · `vigil-rules` · `vigil-module` · `vigil-view`
- `serde`, `serde_json`

## Where it fits

```text
vigil-model   vigil-collect   vigil-rules   vigil-module   vigil-view
     └──────────────┴──────────────┼──────────────┴──────────────┘
                                   ▼
                            vigil-resources
                                   │
                   ┌───────────────┴───────────────┐
                   ▼                               ▼
          vigild · reads and judges        vigil · draws its screen
```
