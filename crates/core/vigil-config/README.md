# vigil-config

> The configuration file as a file: suppressions, collector blocks, and one safe way to write them.

## Why

The daemon and the console both read and write the suppressions — `vigil suppress`, a key on the
findings screen, `vigild configure`, `vigild collector`. One crate holds the write they share: a
temporary file, a rename, the previous version kept as `.previous`, mode `0600`. Both programs
write byte for byte the same way.

## What's inside

| | |
|---|---|
| **Suppression** | one statement of what the host is expected to do, with a `reason` and an optional `until` |
| **Blocks** | the collector blocks under the collectors directory |
| **Safe write** | temporary file, rename, `.previous`, `0600` |

### A suppression entry

An entry names at least one of these, and every condition it names must hold:

- **the object** — `finding_key` (exactly), `finding_key_prefix` (the key begins with it) or
  `finding_key_match` (the whole key fits a pattern);
- **`kind`** — the kind of finding; alone, it covers that kind anywhere on the host;
- **`program`** — a pattern over the program a finding is about: its file name when the pattern
  has no `/`, its full path when it has one.

A pattern has one wildcard, `*`, which stands for any run of characters. A pattern of stars alone
is refused — an entry meant for a whole kind says the kind.

`until` is a date (`2026-12-31`, the start of that day in UTC) or an RFC 3339 moment with `Z` or an
offset. It is read with the file and turned into the moment the agent compares with. Any other
form makes the entry an error.

A tunnel that opens a new UDP port every few minutes is one entry:

```yaml
suppressions:
  - program: netbird
    finding_key_match: "port.listen|udp|*"
    reason: the vpn mesh moves its ports
```

## Depends on

- `serde`, `yaml_serde`

## Where it fits

```text
                  ┌──► vigild · loads, follows the files, judges every finding
vigil-config ─────┤
                  └──► vigil  · vigil suppress, the findings screen, the silenced list
```
