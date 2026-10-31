# vigil-network

> The sockets this host listens on, the program behind each one, and the conversations it holds.

## Why

A listening socket is a door into this host, and a new door is worth a finding. The agent
reads the sockets from the kernel's own tables (`/proc/net` on Linux) and names the program that
holds each one. A program that moves its ports is then one suppression entry, written by name.

Established connections are kept as pairs: a program and the peer it talks to, outbound and
inbound, with loopback set aside.

## What's inside

| | |
|---|---|
| **Reads** | listening TCP, UDP and unix sockets; established connections as program–peer pairs |
| **Raises** | `port.listen.*`, `connection.from_writable_path` |
| **Every** | 30 seconds |
| **Settings** | the `network` block, shipped in `config/collectors/network.yaml` with a comment above every key |

## Depends on

- `vigil-model` · `vigil-collect` · `vigil-rules` · `vigil-module` · `vigil-view`
- `serde_json`; `libc` on macOS

## Where it fits

```text
vigil-model   vigil-collect   vigil-rules   vigil-module   vigil-view
     └──────────────┴──────────────┼──────────────┴──────────────┘
                                   ▼
                             vigil-network
                                   │
                   ┌───────────────┴───────────────┐
                   ▼                               ▼
          vigild · reads and judges        vigil · draws its screen
```
