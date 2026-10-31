# vigil-containers

> The containers running on this host, what they may do and what of the host they hold.

## Why

Containers are read from `/proc`: cgroups, capabilities and mount tables. The agent learns what
runs here without talking to anything that can start a container.

The sockets of the container runtimes are read as files, by mode and owner. A socket anyone may
write to hands the host over. One list of sockets serves both the reading and the rule, and one
socket reached by two paths is one row.

## What's inside

| | |
|---|---|
| **Reads** | containers from `/proc`; runtime sockets by mode and owner |
| **Raises** | `container.privileged`, `container.host_mount`, `container.docker_socket_exposed`, … |
| **Every** | 60 seconds |
| **Settings** | the `containers` block, shipped in `config/collectors/containers.yaml` with a comment above every key |

## Depends on

- `vigil-model` · `vigil-collect` · `vigil-rules` · `vigil-module` · `vigil-view`
- `serde_json`

## Where it fits

```text
vigil-model   vigil-collect   vigil-rules   vigil-module   vigil-view
     └──────────────┴──────────────┼──────────────┴──────────────┘
                                   ▼
                            vigil-containers
                                   │
                   ┌───────────────┴───────────────┐
                   ▼                               ▼
          vigild · reads and judges        vigil · draws its screen
```
