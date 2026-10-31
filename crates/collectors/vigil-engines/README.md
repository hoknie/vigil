# vigil-engines

> The images, volumes, networks, compose projects, pods and secrets the container engines hold.

## Why

A docker or podman client is root on this host in one step, and the daemon starts no program of
its own. So `vigil-container-dump` asks the engines, run by `vigil-containers.timer` (on macOS,
the launchd job `vigil.containers`), and this crate reads the documents it writes.

Each document names the moment of the one before it. A reading counts as stale after two of the
periods the documents show.

## What's inside

| | |
|---|---|
| **Reads** | `/var/lib/vigil/containers/docker.json` and `podman.json` |
| **Raises** | `container.image.*`, `container.volume.*`, `container.network.*`, `container.project.*`, … |
| **Every** | 120 seconds, the period of the dump |
| **Settings** | the `containers-engines` block, shipped in `config/collectors/containers.yaml` with a comment above every key |

## Depends on

- `vigil-model` · `vigil-collect` · `vigil-rules` · `vigil-module` · `vigil-view`
- `serde`, `serde_json`. `vigil-container-dump` links this crate for the document it writes

## Where it fits

```text
vigil-model   vigil-collect   vigil-rules   vigil-module   vigil-view
     └──────────────┴──────────────┼──────────────┴──────────────┘
                                   ▼
                             vigil-engines
                                   │
                   ┌───────────────┴───────────────┐
                   ▼                               ▼
          vigild · reads and judges        vigil · draws its screen
```
