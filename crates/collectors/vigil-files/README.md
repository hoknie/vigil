# vigil-files

> The files this host is expected to keep as they are.

## Why

The watch list names the paths that matter. For each of them the agent keeps the content hash,
the permissions and the suid bit, and reports a change in any of them.

A walk has a ceiling of paths per reading and of bytes per file, so the cost stays within the
agent's budget on any host. When a walk reaches a ceiling, the reading says what it left
unwalked.

## What's inside

| | |
|---|---|
| **Reads** | the paths named in `watch_fs.yaml` |
| **Raises** | `file.*` |
| **Every** | 300 seconds |
| **Settings** | the `files` block, shipped in `config/collectors/files.yaml` with a comment above every key |

## Depends on

- `vigil-model` · `vigil-collect` · `vigil-rules` · `vigil-module` · `vigil-view`
- `serde`, `serde_json`, `yaml_serde` for the watch list; `libc` on macOS

## Where it fits

```text
vigil-model   vigil-collect   vigil-rules   vigil-module   vigil-view
     └──────────────┴──────────────┼──────────────┴──────────────┘
                                   ▼
                              vigil-files
                                   │
                   ┌───────────────┴───────────────┐
                   ▼                               ▼
          vigild · reads and judges        vigil · draws its screen
```
