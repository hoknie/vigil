# vigil-report

> Where findings go once they are made.

## Why

Every destination is a `Reporter`. Writing a file, speaking syslog and posting to a receiver are
the same act for the watch round: hand over a batch and learn whether it arrived. The integration
with a control plane is another reporter.

Delivery is told plainly: a `Delivery` says what was taken and a `ReportError` says why the rest
was refused. The daemon keeps what did not arrive for the next attempt.

## What's inside

| | |
|---|---|
| **`Reporter`** | the port |
| **`NdjsonSink`** | one finding per line, in a file created owner-only whatever the umask |
| **`SyslogSink`** | RFC 5424 messages with structured data to the local syslog socket |
| **`Delivery`, `ReportError`** | what arrived and why the rest did not |

A network sink is still to come. A `webhook` or `host_findings` receiver in the configuration stops
the daemon at start-up with a reason. The buffer of each receiver lives in `vigil-store`.

## Depends on

- `vigil-model`
- `serde_json`

## Where it fits

```text
vigil-model ──► vigil-report ──► vigild · one reporter per receiver
                                    ▲
               vigil-store ─────────┘ · the outgoing buffer of each receiver
```
