# vigil-audit-plugin

> The program auditd starts to hand this host's audit records to the agent.

## Why

On Linux every launch is recorded by the kernel through auditd, and auditd hands its records to a
plugin on standard input. This is that plugin. It makes no decision, opens no socket and reads no
configuration: it appends what it receives to one file, the spool, and the daemon reads the spool
on its own schedule.

The spool has a ceiling. When it is full the oldest records give way, the disk stays free, and the
daemon reports the loss as a finding.

## What's inside

| | |
|---|---|
| **Registered in** | `/etc/audit/plugins.d/vigil.conf`, shipped with the package |
| **`--spool PATH`** | where to write, `/var/lib/vigil/audit-spool` by default |
| **`--ceiling BYTES`** | how much the spool may hold, 16 MiB by default |
| **By hand** | replays a recorded log: `vigil-audit-plugin --spool /tmp/spool < audit.log` |

## Depends on

- `vigil-launches` — the spool writer and the defaults it shares with the collector

## Where it fits

```text
kernel ──► auditd ── stdin ──► vigil-audit-plugin ──► spool ──► vigild · launches
                                       ▲
                               vigil-launches · SpoolWriter
```
