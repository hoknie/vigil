# vigil-launches-spool

A program of the macOS package. It keeps `/usr/bin/eslogger exec --format json` running and
writes every program a person launched on this Mac to the spool
`/usr/local/var/lib/vigil/launches/exec-spool`, in the record format the launches collector
reads for auditd.

## The decision it follows from

The daemon starts no program, and Endpoint Security needs root and Full Disk Access. So the
privileged, always-running half of the launches reading is a launchd job — `vigil.launches`,
as root, `KeepAlive` — as `vigil-audit-plugin` is auditd's on Linux, and it hands the daemon a
file rather than a socket.

## What it writes, and what it never writes

For every exec event of a new image whose audit user is set — a person logged in for it —
two records: the moment and eslogger's global sequence, the pid, the parent, the audit user,
the real and effective user, and the program; then how many arguments there were. The
arguments themselves only when `record_arguments` is on in the `launches` block of the
configuration (`--configuration`, `/usr/local/etc/vigil/vigil.yaml`, and the collectors
directory it names), read once at start, and then after `vigil_collect::redact` has hidden
what it recognises: nothing is written that was not redacted first. A configuration that
cannot be read means no arguments. The environment eslogger prints is never written. A launch
no person logged in for is not written.

Beside the spool it keeps `eslogger.json`, the state of eslogger: `running`, `refused` with
what eslogger said, `stopped` with its last words, `absent` on a Mac without eslogger. The
collector turns it into its health line. The job exits when eslogger does, and launchd starts
it again after 30 seconds.

What one read of eslogger's pipe holds (256 KiB at most) is written in one write: every write
reaches the disk before it returns, and one write per launch is a Mac that compiles something
waiting on its own audit.

## Dependencies

`vigil-launches` (the parser of eslogger's JSON, the records, the spool writer and the status),
`vigil-config` (the configuration and its collectors directory), `serde_yaml`.

## Context

```
vigil.launches (launchd) ──► vigil-launches-spool ──► exec-spool + eslogger.json ──► vigil-launches ──► vigild
                                  └── /usr/bin/eslogger exec --format json (root, Full Disk Access)
```
