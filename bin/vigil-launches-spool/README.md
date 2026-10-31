# vigil-launches-spool

> Keeps `eslogger` running and writes every program a person launches on this Mac to the spool.

## Why

Endpoint Security needs root and Full Disk Access, and the daemon starts no program. So the
privileged, always-running half of the launches reading on macOS is a launchd job — `vigil.launches`,
as root, `KeepAlive`. It plays the part `vigil-audit-plugin` plays for auditd on Linux and hands the
daemon a file: `/usr/local/var/lib/vigil/launches/exec-spool`, in the record format the launches
collector reads.

## What's inside

| | |
|---|---|
| **Writes** | for each exec of a new image with an audit user set: the moment and sequence, pid, parent, audit user, real and effective user, program, and the number of arguments |
| **Arguments** | written only when `record_arguments` is on in the `launches` block, and always after `vigil_collect::redact`; an unreadable configuration means no arguments |
| **Leaves out** | the environment eslogger prints, and launches no person logged in for |
| **Status** | `eslogger.json` beside the spool: `running`, `refused`, `stopped` or `absent` — the collector shows it as its health |
| **Writes in bulk** | one read of eslogger's pipe (256 KiB at most) is one write |

The job exits when eslogger does, and launchd starts it again after 30 seconds.

## Depends on

- `vigil-launches` — the eslogger parser, the records, the spool writer and the status
- `vigil-config` — the configuration and its collectors directory
- `yaml_serde`

## Where it fits

```text
vigil.launches · launchd ──► vigil-launches-spool ──► exec-spool + eslogger.json ──► vigild
                                     │
                     /usr/bin/eslogger exec --format json
```
