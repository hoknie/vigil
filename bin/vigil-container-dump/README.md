# vigil-container-dump

The fourth program of the package. It runs the listing commands of this host's container
engines and writes what each one printed, whole, `0600`, to
`/var/lib/vigil/containers/docker.json` and `.../podman.json`.

## The decision it follows from

The daemon starts no program of its own, and a docker or podman client is root on this host in
one step. So the privileged half of this reading is a unit an operator can read and switch
off — `systemctl mask vigil-containers.timer` — exactly as `vigil-firewall.service` is for
`nft` (`docs/designs/2026-09-17-DESIGN-containers-engines.md`).

## What it runs

Nothing of it comes from the host. For docker: `image ls --digests`, `volume ls`,
`network ls`, `ps -a`, `system info`; for podman the same plus `pod ls` and `secret ls` — all
with `--format json`, all through `vigil-engines`, where the list is written down and a test
refuses any word that is not `ls`, `ps` or `info`. The client is found by absolute path
(`/usr/bin`, `/usr/local/bin`, `/bin`), run with `env_clear()` and a fixed `PATH`, never
through a shell.

Each command gets its own deadline (`--deadline`, ten seconds by default): its output goes to a
scratch file rather than a pipe, so a chatty engine cannot wedge it, and a command that has not
finished by the deadline is killed and written down as `timed_out`. Output over `--ceiling`
(8 MiB) is cut and the answer says `truncated`.

## What it writes

One document per engine, built as `serde_json` and renamed into place, so a reader never sees
half a dump:

- an engine that is not installed → `"state": "absent"` with the paths it looked in. That is
  not the same document as an engine that failed, and the reader must be able to tell
  (trap 1 in `CLAUDE.md`);
- an engine that is installed → `"state": "present"`, the program it ran, and one entry per
  subject with `state` (`answered`, `failed`, `timed_out`), the argument list, how long it
  took, what it printed verbatim, and whatever the engine wrote on standard error.

It decides nothing else. Projection, stabilisation and the hiding of secrets are in
`vigil-engines`, where a fixture pins them.

## macOS

The launchd job `vigil.containers` runs it as root every 120 seconds, writing to
`/usr/local/var/lib/vigil/containers`. The clients are found at the paths a Mac has them
(`vigil-engines` lists them), and there is one rule of its own: **a client is run as the
account that owns its file.** Docker Desktop is dragged into `/Applications` by a person and
belongs to them, and so does Homebrew's `/opt/homebrew`; run as root, the client they can
replace would hand them root the next time the job ran. So before a client runs, every
directory from `/` to it is checked: each must belong to root or to the owner of the client,
and none may be writable by any account, nor by a group other than `wheel` and `admin` (whose
members may become root already). A client that passes runs with that owner's uid, gid and
`HOME`; one that does not is not run, and every subject is written down as `failed` with the
reason. The document names the account in `account`. Started by hand by an account that is not
root, every client runs as that account.

## Dependencies

`vigil-engines` for the vocabulary the writer and the reader share — the engines, the command
list, the document — `serde_json`, and `libc` on macOS for the account a client belongs to.
No socket, no configuration, no findings.

## Context

```
vigil-containers.timer ──► vigil-container-dump ──► /var/lib/vigil/containers/*.json
                                    │                             │
                                    └── vigil-engines ────────────┴──► vigild
```
