# vigil-firewall-dump

A program of the macOS package. It asks pf and the Application Firewall of this Mac what they
hold and writes what each answered, whole, `0600`, to
`/usr/local/var/lib/vigil/firewall/firewall.json`.

## The decision it follows from

The daemon starts no program, and `pfctl` answers root alone. So the privileged half of the
firewall reading is a launchd job an operator can read and switch off — `vigil.firewall`,
every 60 seconds, as root — exactly as `vigil-firewall.timer` runs `nft` on Linux. On Linux
this program says so and writes nothing.

## What it runs

Nothing of it comes from the Mac except the names of pf's anchors, which are checked to be
names pf could have given (letters, digits, `._-/`, no leading `-` or `/`, no `..`) before
one becomes an argument:

```
/sbin/pfctl -s info
/sbin/pfctl -s rules
/sbin/pfctl -s nat
/sbin/pfctl -v -s Anchors
/sbin/pfctl -a <anchor> -s rules     for each anchor listed, 64 at most
/sbin/pfctl -a <anchor> -s nat
/usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate --getblockall
    --getstealthmode --getallowsigned --listapps
```

The list is in `vigil-firewall` (`types::Question`), and a test refuses any flag that switches,
loads, flushes or kills. `-v` is not given to `-s rules`: it adds the counters of every rule,
which move on every run. Each command runs by its absolute path, with an empty environment,
never through a shell, with a deadline of its own (`--deadline`, ten seconds), its output going
to a scratch file rather than a pipe and cut at `--ceiling` (4 MiB).

## What it writes

One document, built as `serde_json` and renamed into place: the time, the program that wrote
it, and one answer per question with `state` (`answered`, `failed`, `timed_out`), the program
and arguments, how long it took, what it printed verbatim, and what it said on standard error.
It decides nothing else: `vigil-firewall` parses the answers, on the daemon's round.

Run by hand without root it still writes the document: the Application Firewall answered and
every `pfctl` question failed with "Permission denied", which the collector reads as "pf is
unknown", not as "pf filters nothing". Release, on this Mac, unprivileged: 0.17 s, 2.5 MB.

## Dependencies

`vigil-firewall` for the questions and the document, and `serde_json`.

## Context

```
vigil.firewall (launchd) ──► vigil-firewall-dump ──► /usr/local/var/lib/vigil/firewall/firewall.json
                                  │                                   │
                        (/sbin/pfctl, socketfilterfw)                 ▼
                                  └────── vigil-firewall ◄──────── bin/vigild
```
