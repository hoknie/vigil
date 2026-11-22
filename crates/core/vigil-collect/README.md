# vigil-collect

> What reading a host means, and the pieces every reading shares.

## Why

A collector tells "could not read" apart from "read, and found nothing". `/proc` changes shape with
the kernel, a container sees a slice of it, a pid namespace hides part of it. So `collect` returns a
`Snapshot` or a `CollectError`, and `available` returns a `Health` that the daemon reports as it is.
A failed reading is always visible as a failure.

A collector also says what it is not shown. An agent run by an ordinary account reads everything
that account may read, and the operating system keeps the rest for root: the sockets and command
lines of other accounts, the crontabs, `/etc/sudoers`. That is a `Limit`, a list of what is not
seen, and it is told beside the `Health` in one `Standing`. A limit is the mode the agent runs in;
a `Degraded` health is a reading that went wrong. A refusal counts as a limit only when the agent
is not root (`denied_to_this_account`): what root is refused is a failure.

Each subject crate does its own reading. This crate holds what several of them share.

## What's inside

| | |
|---|---|
| **`Collector`** | the port: `name`, `available`, `standing`, `collect` |
| **Health** | `Health`, `CollectError`, and `NotOnThisSystem` for a subject this OS does not have |
| **Limits** | `Limit`, `Standing`; on macOS `denied_to_this_account`, `denied_to` and `named_account` |
| **Redaction** | `redact` and `Redacted` — secrets in command lines are hidden before anything is kept |
| **Hashing** | `sha256`, `hex` |
| **Formats** | `/proc` escapes (`unescaped`), command lines (`split_command`), `/etc/passwd` (`parse_passwd`) |
| **macOS** | `sysctl`, the process table and process arguments |

## Depends on

- `vigil-model`
- `serde_json`, `sha2`

## Where it fits

```text
vigil-model ──► vigil-collect ──► collectors/vigil-<subject>/collectors
                     │
                     └─────────► vigil-module · the collector a module hands the daemon
```
