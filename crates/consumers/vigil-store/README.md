# vigil-store

What the agent keeps on its own disk: the snapshot each collector last read, the history of
findings, and the findings each receiver has not taken yet.

## The decision it follows from

The local store is files, not a database:
an append-only NDJSON journal of findings and one JSON snapshot per collector, so a restart
continues watching instead of learning the host again, and history is read at an incident with
`cat` and `jq`. Every write is atomic, durable and `0600`, and a damaged file costs what it held
rather than the start of the daemon.

The `Store` port and the conformance suite in `conformance` say what any backend must do; the
files backend behind the `files` feature is the one there is.

## The files under the state directory

- `findings/journal.ndjson` — the journal of findings, one per line;
- `baselines/<collector>.json` — the last reading of each collector;
- `outgoing/<receiver>.ndjson` — the findings a receiver has not taken, oldest first, under a
  ceiling of findings and of bytes that displaces the oldest rather than refusing the newest;
- `outgoing/<receiver>.batch` — the batch a receiver refused, while there is one: its
  `batch_id`, how many findings from the head of the buffer it holds, and the `event_id` of the
  first and the last of them. The next attempt, in this run or after a restart, sends exactly
  that batch under that id, so a receiver can tell a retry from a new batch.
  It is written only on a refusal and removed when the batch is taken, when no retry can fix it,
  or when the buffer no longer starts with those findings — the ceiling displaced some, or the
  files no longer agree when the buffer is opened. A delivery taken at once never writes it.

`create_owner_only` and `create_owner_only_directory` are exported for the one other file the
daemon writes findings to, the `ndjson` receiver's own.

## Dependencies

`vigil-model`, `serde_json`. No io beyond `std::fs`.

## Context

```
vigil-model ──► vigil-store ──► bin/vigild  (the watch cycle, delivery, the ndjson receiver)
```
