# vigil-firewall-dump

> Asks pf and the Application Firewall of this Mac what they hold and writes their answers to disk.

## Why

`pfctl` answers root alone, and the daemon starts no program. So the privileged half of the
firewall reading on macOS is a launchd job an operator can read and switch off — `vigil.firewall`,
every 60 seconds, as root. On Linux `vigil-firewall.timer` plays the same part with `nft`, and this
program says so and writes nothing.

## What's inside

| | |
|---|---|
| **Asks** | `pfctl -s info`, `-s rules`, `-s nat`, `-v -s Anchors`, then rules and nat for each anchor (64 at most); `socketfilterfw` for its global state and app list |
| **Safety** | the questions live in `vigil-firewall`, where a test refuses any flag that switches, loads, flushes or kills; anchor names are checked before they become arguments; each command runs by absolute path with an empty environment, outside any shell |
| **Limits** | a deadline per command (`--deadline`, 10 s) and a ceiling on output (`--ceiling`, 4 MiB) |
| **Writes** | one document, `0600`, renamed into place: `/usr/local/var/lib/vigil/firewall/firewall.json` |

Each answer carries `state` (`answered`, `failed`, `timed_out`), the program and arguments, the
time it took, what it printed and what it said on standard error. `vigil-firewall` parses the
answers on the daemon's round.

Run by hand without root, the Application Firewall still answers and every `pfctl` question fails
with "Permission denied"; the collector reads that as "pf is unknown". On a Mac, unprivileged, a run
takes 0.17 s and 2.5 MB.

## Depends on

- `vigil-firewall` — the questions and the document
- `serde_json`

## Where it fits

```text
vigil.firewall · launchd ──► vigil-firewall-dump ──► firewall.json
                                   │                       │
                       pfctl · socketfilterfw              ▼
                                   └──── vigil-firewall ◄── vigild
```
