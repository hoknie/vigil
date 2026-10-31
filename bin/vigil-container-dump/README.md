# vigil-container-dump

> Asks this host's container engines what they hold and writes their answers to disk.

## Why

A docker or podman client is root on this host in one step, and the daemon starts no program of
its own. So the privileged half of the engines reading is a unit an operator can read and switch
off — `systemctl mask vigil-containers.timer` — the same arrangement as `vigil-firewall.service`
for `nft`.

## What's inside

| | |
|---|---|
| **Runs** | for docker: `image ls --digests`, `volume ls`, `network ls`, `ps -a`, `system info`; for podman the same plus `pod ls` and `secret ls` — all with `--format json` |
| **Safety** | the list lives in `vigil-engines`, where a test accepts only `ls`, `ps` and `info`; the client is found by absolute path and run with an empty environment and a fixed `PATH`, outside any shell |
| **Limits** | a deadline per command (`--deadline`, 10 s) and a ceiling on output (`--ceiling`, 8 MiB); output goes to a scratch file |
| **Writes** | one document per engine, `0600`, renamed into place: `/var/lib/vigil/containers/docker.json` and `podman.json` |

### The document

- An engine that is installed: `"state": "present"`, the program it ran, and one entry per subject
  with `state` (`answered`, `failed`, `timed_out`), the arguments, the time it took, what it
  printed and what it wrote on standard error.
- An engine that is missing: `"state": "absent"` with the paths it looked in. A missing engine and
  a failed one are two distinct documents.
- `previous_taken_at` names the moment of the document it replaces, so the agent learns the period
  from the documents themselves.

Projection, stabilisation and hiding secrets happen in `vigil-engines`.

### macOS

The launchd job `vigil.containers` runs it as root every 120 seconds, writing to
`/usr/local/var/lib/vigil/containers`. Each client runs as the account that owns its file: Docker
Desktop in `/Applications` and Homebrew's `/opt/homebrew` belong to a person. Before a client runs,
every directory from `/` to it is checked — each belongs to root or to the client's owner, and only
`wheel` and `admin` may write to it as a group. A client that passes runs with its owner's uid, gid
and `HOME`. A client that fails the check is recorded as `failed` with the reason.

## Depends on

- `vigil-engines` — the engines, the command list and the document
- `serde_json`; `libc` on macOS

## Where it fits

```text
vigil-containers.timer ──► vigil-container-dump ──► /var/lib/vigil/containers/*.json
                                    │                              │
                                    └── vigil-engines ─────────────┴──► vigild
```
