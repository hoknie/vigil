# vigil-config

The configuration file as a file: what `vigil.yaml` points at, the suppressions both binaries
read and write, the collector blocks, and the one safe `0600` write they all go through.

## The decision it follows from

The daemon and the console read suppressions from the same `suppressions_path` and both write
them (`vigil suppress`, a key on the findings screen; `vigild configure` and `vigild collector`
write the file too). Without one crate for it, the same write — a temporary file, a rename, the
previous version kept as `.previous` — would live in two binaries that must stay byte-identical
and would drift apart on the first edit. That is why this crate is the one place in `core/` that
touches files (`ARCHITECTURE.md` §10).

## A suppression entry

One entry is one statement of what the host is expected to do, always with a `reason`, and
optionally an `until` after which it stops covering anything. `until` is a date (`2026-12-31`,
the start of that day in UTC) or an RFC 3339 moment with `Z` or an offset
(`2026-12-31T03:00:00+03:00`). It is read when the file is read and turned into the moment the
agent compares with (`2026-12-31T00:00:00.000Z`); a day that does not exist, or any other form,
makes the entry an error rather than one that never expires. The daemon and the console read it
with the same function. An entry names at least one of:

- the object, in one of three ways — `finding_key` (exactly), `finding_key_prefix` (the key
  begins with it) or `finding_key_match` (the whole key fits a pattern);
- `kind` — the kind of finding; alone, that kind anywhere on the host;
- `program` — a pattern over the program the finding is about: its file name when the pattern
  has no `/`, its full path when it has one. A finding with no program is never covered.

Every condition an entry names must hold. A pattern has one wildcard, `*`, which stands for any
run of characters, `/` and `|` included; everything else is compared as written, case included.
A pattern of stars alone is refused: it would silence everything the other conditions allow,
and an entry that means a kind should say the kind.

A tunnel that opens a new UDP port every few minutes is one entry:

```yaml
suppressions:
  - program: netbird
    finding_key_match: "port.listen|udp|*"
    reason: the vpn mesh moves its ports
```

The program of a finding is decided in `vigil-model` (`Finding::program`), once, for the daemon
that silences and the console that hides.

## Dependencies

`serde`, `yaml_serde`. No crate of ours.

## Context

```
                    ┌──► bin/vigild  (loads, follows the files, judges every finding)
vigil-config ───────┤
                    └──► bin/vigil   (vigil suppress, the findings screen, the silenced list)
```
