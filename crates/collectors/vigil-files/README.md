# vigil-files

The files this host is configured by: whether their content, their mode or their owner moved,
whether a program became one that runs as its owner, whether a file appeared in or left a
directory the operator asked to watch, and whether a directory on `PATH` is one anybody may
drop a program into.

## The decision it follows from

A file watcher that hashes everything is the first agent an operator turns off (trap 5 in
`CLAUDE.md`). So nothing is walked unless a list names it, every walk is bounded, and the cost
of the bound is measured rather than guessed (below). The list is the operator's: it lives in a
file of its own, the collector reads it again on every reading, and the console edits it one
line at a time.

## Settings

The block the daemon hands this module (`/etc/vigil/collectors/files.yaml`, with `enabled` and
`schedule` already taken out) has two layouts, and a block holds one of them. Mixing the keys of
both is refused at start-up in a sentence naming both, because one block with two lists is a
block where the agent watches one list and the file reads as the other.

| Key | Layout | Meaning |
|---|---|---|
| `watched_path` | list | absolute path of the watch list: one file, or a directory whose `*.yaml` / `*.yml` files are read in the order of their names (the rules of `vigil_config::files_in`) |
| `max_file_size` | list | bytes, or `512kb`, `30mb`, `1gb` (each 1024 of the one before); a file past it is watched by its mode and owner and not hashed. Refused above 64mb |
| `devices` | list | `include` / `exclude`: filesystems a walk may step into (below) |
| `max_files` | list | paths walked in one reading, default and most 10000 |
| `paths` | old | the list inside `vigil.yaml`, files only, at most 256 |
| `ceiling_bytes` | old | bytes, what `max_file_size` is in the list layout |

A block with neither `watched_path` nor `paths` watches the six paths this product ships
(`WATCHED_BY_DEFAULT`), as it always did.

## The watch list

```yaml
files:
  - /etc/ssh/sshd_config          # a file
  - /etc/pam.d                    # a directory, walked whole
  - /etc/ssh/*.conf               # a mask: *, ?, [...] and [!...] inside one name
  - path: /etc/ssl/certs/ca.crt
    max_file_size: 8mb
devices:
  include: []
  exclude: []
```

`parsers::watch_list_in` reads it with `deny_unknown_fields`, the same checks as the block, and
the place of a refused entry (`files #3`). `collectors::lists::Lists` holds each list file with
the `stat` it was read at (device, inode, size, both times to the nanosecond) and parses it again
only when that moved, so a list nobody touched costs one `stat` a reading. An edit applies at
the next reading, with nothing asked of the daemon. A list that stops parsing keeps the list it
held before and makes the reading `degraded` with the file named; a list that never parsed, a
list that is not there, and a directory with no list in it are each said in their own sentence,
and none of them is ever a reading of nothing.

## Walks

- A directory is walked depth first in the order of names, so a walk cut at `max_files` is cut
  at the same place on every reading, and the paths past it do not come and go.
- A link is a row saying where it points (`target`), never followed; a mask's fixed prefix is
  followed, as a named path is.
- The kernel's own filesystems are never entered (`proc`, `sysfs`, `devpts`, `devtmpfs`,
  `cgroup`, `cgroup2`, `securityfs`, `debugfs`, `tracefs`, `bpf`, `mqueue`, `hugetlbfs`, `pstore`,
  `configfs`, `fusectl`, `autofs`, `binfmt_misc`, `rpc_pipefs`, `nsfs`, `efivarfs`, `selinuxfs`).
- `devices`: an empty `include` is every other filesystem; an entry names a mount by its source
  device (`/dev/sda1`), its mount point (`/mnt/nfs`) or its kind (`nfs4`); `exclude` wins; the
  list's `devices` is added to the block's. Mounts are read from `/proc/self/mountinfo`
  (`parsers::mountinfo`, `\040` escapes read back); a step into another filesystem is seen by a
  changed device number or by a path that is a mount point, which is what catches a bind mount.
  With no mount table, no walk leaves the filesystem it started on, and the health says so.
- Deeper than 64 directories is not entered. What was not entered is on the entry's row.
- A mask's `*` matches a name beginning with a dot: a hidden file dropped into a watched place
  is the first thing a watcher must not miss.
- A name that is not UTF-8 is not a row (a row key is text); it is listed as not entered, so it
  is seen rather than skipped.

## The reading

One snapshot, source `files`:

| Key | Row |
|---|---|
| `file\|<path>` | a watched path: `present`, `readable`, `sha256`, `size`, `mode`, `uid`, `gid`, `over_the_ceiling`, `ceiling_bytes`; a walked row adds `type`, `found_by`, `complete`, and a link `target` |
| `directory\|<path>` | a directory on `PATH` |
| `walk\|<entry>` | a directory or mask of the list: `kind` (`tree`, `mask`), `matched`, `complete`, `not_entered`, `max_file_size` |

A file named in the list is keyed and valued exactly as it was before walks existed, and a test
says so field by field: a host whose list did not change keeps its baseline.

## Findings

No new kind. `file.changed` already says a watched file came or went, and a file that appears in
a watched directory or leaves it is that fact. `rules::FileAppearedOrGone` is a batch rule over
the rows a walk found:

- an entry just added to the list, or taken off it, says nothing about what it brings into view
  or out of it — the `walk|` row arrived or left in the same batch;
- a walk cut at `max_files` on either reading, or one whose `not_entered` moved, says nothing
  about paths coming and going: the paths moved across the limit or the operator's `devices`,
  not on the host;
- a directory that appears or goes with paths under it is one finding that counts them.

A walked directory gaining the sgid bit is a mode that changed, not a program that runs as its
owner. A link pointing somewhere else is `file.changed`.

## The console

`views` draws every row; a `walk|` row says `matches nothing` for a mask that matched nothing.
The form calls the per-path size `max file size` and reads `8mb` as well as bytes. A path found
by a walk is changed through the entry that found it. Where the console writes is
`bin/vigil/src/config/watched`: `console.yaml` in the `watched_path` directory, or that file.

## Cost

Measured with `cargo run --release -p vigil-files --example files` in the Linux container
(`just docker`), 2026-09-18, files of 0.5–4 KB, 20 subdirectories:

| Reading | Before | After | Rows | Baseline | Most resident |
|---|---|---|---|---|---|
| six shipped paths, from the block | 0.06 ms | 0.06 ms | 12 | 2 KB | 0.8 MB |
| six paths, from a watch list | — | 0.05 ms | 12 | 2.5 KB | 0.9 MB |
| a directory of 1000 files | — | 7–8 ms | 1028 | 415 KB | 7 MB |
| a directory of 10000 files | — | 107–110 ms | 10008 | 4.2 MB | 62 MB |

A reading every 300 s of ten thousand paths is 0.04 % of one core; its memory is the whole
agent's budget (`ARCHITECTURE.md` §3.7), which is why `max_files` refuses more than 10000.

## Dependencies

`vigil-model`, `vigil-collect`, `vigil-rules`, `vigil-module`, `vigil-view`, `serde`,
`serde_json`, `serde_yaml`. No new crate: masks are matched here, in `helpers::mask`.

```
bin/vigild ──┐
             ├──► vigil-files ──► vigil-module · vigil-rules · vigil-collect · vigil-view · vigil-model
bin/vigil  ──┘         ▲
                       └── /etc/vigil/watch_fs.yaml, /proc/self/mountinfo, the watched paths
```
